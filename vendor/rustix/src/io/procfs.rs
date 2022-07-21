//! Utilities for working with `/proc`, where Linux's `procfs` is typically
//! mounted. `/proc` serves as an adjunct to Linux's main syscall surface area,
//! providing additional features with an awkward interface.
//!
//! This module does a considerable amount of work to determine whether `/proc`
//! is mounted, with actual `procfs`, and without any additional mount points
//! on top of the paths we open.
//!
//! Why all the effort to detect bind mount points? People are doing all kinds
//! of things with Linux containers these days, with many different privilege
//! schemes, and we want to avoid making any unnecessary assumptions. Rustix
//! and its users will sometimes use procfs *implicitly* (when Linux gives them
//! no better options), in ways that aren't obvious from their public APIs.
//! These filesystem accesses might not be visible to someone auditing the main
//! code of an application for places which may be influenced by the filesystem
//! namespace. So with the checking here, they may fail, but they won't be able
//! to succeed with bogus results.

use crate::fd::{AsFd, BorrowedFd};
use crate::ffi::ZStr;
use crate::fs::{
    cwd, fstat, fstatfs, major, openat, renameat, Dir, FileType, Mode, OFlags, Stat,
    PROC_SUPER_MAGIC,
};
use crate::io::{self, OwnedFd};
use crate::path::DecInt;
use crate::process::{getgid, getpid, getuid, Gid, RawGid, RawUid, Uid};
#[cfg(feature = "rustc-dep-of-std")]
use core::lazy::OnceCell;
#[cfg(not(feature = "rustc-dep-of-std"))]
use once_cell::sync::OnceCell;

/// Linux's procfs always uses inode 1 for its root directory.
const PROC_ROOT_INO: u64 = 1;

// Identify an entry within "/proc", to determine which anomalies to
// check for.
#[derive(Debug)]
enum Kind {
    Proc,
    Pid,
    Fd,
    File,
}

/// Check a subdirectory of "/proc" for anomalies.
fn check_proc_entry(
    kind: Kind,
    entry: BorrowedFd<'_>,
    proc_stat: Option<&Stat>,
    uid: RawUid,
    gid: RawGid,
) -> io::Result<Stat> {
    let entry_stat = fstat(&entry)?;
    check_proc_entry_with_stat(kind, entry, entry_stat, proc_stat, uid, gid)
}

/// Check a subdirectory of "/proc" for anomalies, using the provided `Stat`.
fn check_proc_entry_with_stat(
    kind: Kind,
    entry: BorrowedFd<'_>,
    entry_stat: Stat,
    proc_stat: Option<&Stat>,
    uid: RawUid,
    gid: RawGid,
) -> io::Result<Stat> {
    // Check the filesystem magic.
    check_procfs(entry)?;

    match kind {
        Kind::Proc => check_proc_root(entry, &entry_stat)?,
        Kind::Pid | Kind::Fd => check_proc_subdir(entry, &entry_stat, proc_stat)?,
        Kind::File => check_proc_file(&entry_stat, proc_stat)?,
    }

    // Check the ownership of the directory. We can't do that for the toplevel
    // "/proc" though, because in e.g. a user namespace scenario, root outside
    // the container may be mapped to another uid like `nobody`.
    if !matches!(kind, Kind::Proc) {
        if (entry_stat.st_uid, entry_stat.st_gid) != (uid, gid) {
            return Err(io::Error::NOTSUP);
        }
    }

    // "/proc" directories are typically mounted r-xr-xr-x.
    // "/proc/self/fd" is r-x------. Allow them to have fewer permissions, but
    // not more.
    let expected_mode = if let Kind::Fd = kind { 0o500 } else { 0o555 };
    if entry_stat.st_mode & 0o777 & !expected_mode != 0 {
        return Err(io::Error::NOTSUP);
    }

    match kind {
        Kind::Fd => {
            // Check that the "/proc/self/fd" directory doesn't have any extraneous
            // links into it (which might include unexpected subdirectories).
            if entry_stat.st_nlink != 2 {
                return Err(io::Error::NOTSUP);
            }
        }
        Kind::Pid | Kind::Proc => {
            // Check that the "/proc" and "/proc/self" directories aren't empty.
            if entry_stat.st_nlink <= 2 {
                return Err(io::Error::NOTSUP);
            }
        }
        Kind::File => {
            // Check that files in procfs don't have extraneous hard links to
            // them (which might indicate hard links to other things).
            if entry_stat.st_nlink != 1 {
                return Err(io::Error::NOTSUP);
            }
        }
    }

    Ok(entry_stat)
}

fn check_proc_root(entry: BorrowedFd<'_>, stat: &Stat) -> io::Result<()> {
    // We use `O_DIRECTORY` for proc directories, so open should fail if we
    // don't get a directory when we expect one.
    assert_eq!(FileType::from_raw_mode(stat.st_mode), FileType::Directory);

    // Check the root inode number.
    if stat.st_ino != PROC_ROOT_INO {
        return Err(io::Error::NOTSUP);
    }

    // Proc is a non-device filesystem, so check for major number 0.
    // <https://www.kernel.org/doc/Documentation/admin-guide/devices.txt>
    if major(stat.st_dev.into()) != 0 {
        return Err(io::Error::NOTSUP);
    }

    // Check that "/proc" is a mountpoint.
    if !is_mountpoint(entry) {
        return Err(io::Error::NOTSUP);
    }

    Ok(())
}

fn check_proc_subdir(
    entry: BorrowedFd<'_>,
    stat: &Stat,
    proc_stat: Option<&Stat>,
) -> io::Result<()> {
    // We use `O_DIRECTORY` for proc directories, so open should fail if we
    // don't get a directory when we expect one.
    assert_eq!(FileType::from_raw_mode(stat.st_mode), FileType::Directory);

    check_proc_nonroot(stat, proc_stat)?;

    // Check that subdirectories of "/proc" are not mount points.
    if is_mountpoint(entry) {
        return Err(io::Error::NOTSUP);
    }

    Ok(())
}

fn check_proc_file(stat: &Stat, proc_stat: Option<&Stat>) -> io::Result<()> {
    // Check that we have a regular file.
    if FileType::from_raw_mode(stat.st_mode) != FileType::RegularFile {
        return Err(io::Error::NOTSUP);
    }

    check_proc_nonroot(stat, proc_stat)?;

    Ok(())
}

fn check_proc_nonroot(stat: &Stat, proc_stat: Option<&Stat>) -> io::Result<()> {
    // Check that we haven't been linked back to the root of "/proc".
    if stat.st_ino == PROC_ROOT_INO {
        return Err(io::Error::NOTSUP);
    }

    // Check that we're still in procfs.
    if stat.st_dev != proc_stat.unwrap().st_dev {
        return Err(io::Error::NOTSUP);
    }

    Ok(())
}

/// Check that `file` is opened on a `procfs` filesystem.
fn check_procfs(file: BorrowedFd<'_>) -> io::Result<()> {
    let statfs = fstatfs(&file)?;
    let f_type = statfs.f_type;
    if f_type != PROC_SUPER_MAGIC {
        return Err(io::Error::NOTSUP);
    }

    Ok(())
}

/// Check whether the given directory handle is a mount point. We use a
/// `renameat` call that would otherwise fail, but which fails with `EXDEV`
/// first if it would cross a mount point.
fn is_mountpoint(file: BorrowedFd<'_>) -> bool {
    let err = renameat(&file, zstr!("../."), &file, zstr!(".")).unwrap_err();
    match err {
        io::Error::XDEV => true,  // the rename failed due to crossing a mount point
        io::Error::BUSY => false, // the rename failed normally
        _ => panic!("Unexpected error from `renameat`: {:?}", err),
    }
}

/// Open a directory in `/proc`, mapping all errors to `io::Error::NOTSUP`.
fn proc_opendirat<P: crate::path::Arg, Fd: AsFd>(dirfd: Fd, path: P) -> io::Result<OwnedFd> {
    // We could add `PATH`|`NOATIME` here but Linux 2.6.32 doesn't support it.
    let oflags = OFlags::NOFOLLOW | OFlags::DIRECTORY | OFlags::CLOEXEC | OFlags::NOCTTY;
    openat(dirfd, path, oflags, Mode::empty()).map_err(|_err| io::Error::NOTSUP)
}

/// Returns a handle to Linux's `/proc` directory.
///
/// This ensures that `/proc` is procfs, that nothing is mounted on top of it,
/// and that it looks normal. It also returns the `Stat` of `/proc`.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man5/proc.5.html
fn proc() -> io::Result<(BorrowedFd<'static>, &'static Stat)> {
    static PROC: StaticFd = StaticFd::new();

    // `OnceBox` is "racey" in that the initialization function may run
    // multiple times. We're ok with that, since the initialization function
    // has no side effects.
    PROC.get_or_try_init(|| {
        // Open "/proc".
        let proc = proc_opendirat(&cwd(), zstr!("/proc"))?;
        let proc_stat = check_proc_entry(
            Kind::Proc,
            proc.as_fd(),
            None,
            Uid::ROOT.as_raw(),
            Gid::ROOT.as_raw(),
        )
        .map_err(|_err| io::Error::NOTSUP)?;

        Ok(new_static_fd(proc, proc_stat))
    })
    .map(|(fd, stat)| (fd.as_fd(), stat))
}

/// Returns a handle to Linux's `/proc/self` directory.
///
/// This ensures that `/proc/self` is procfs, that nothing is mounted on top of
/// it, and that it looks normal. It also returns the `Stat` of `/proc/self`.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man5/proc.5.html
fn proc_self() -> io::Result<(BorrowedFd<'static>, &'static Stat)> {
    static PROC_SELF: StaticFd = StaticFd::new();

    // The init function here may run multiple times; see above.
    PROC_SELF
        .get_or_try_init(|| {
            let (proc, proc_stat) = proc()?;

            let (uid, gid, pid) = (getuid(), getgid(), getpid());

            // Open "/proc/self". Use our pid to compute the name rather than literally
            // using "self", as "self" is a symlink.
            let proc_self = proc_opendirat(&proc, DecInt::new(pid.as_raw_nonzero().get()))?;
            let proc_self_stat = check_proc_entry(
                Kind::Pid,
                proc_self.as_fd(),
                Some(proc_stat),
                uid.as_raw(),
                gid.as_raw(),
            )
            .map_err(|_err| io::Error::NOTSUP)?;

            Ok(new_static_fd(proc_self, proc_self_stat))
        })
        .map(|(owned, stat)| (owned.as_fd(), stat))
}

/// Returns a handle to Linux's `/proc/self/fd` directory.
///
/// This ensures that `/proc/self/fd` is `procfs`, that nothing is mounted on
/// top of it, and that it looks normal.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man5/proc.5.html
pub fn proc_self_fd() -> io::Result<BorrowedFd<'static>> {
    static PROC_SELF_FD: StaticFd = StaticFd::new();

    // The init function here may run multiple times; see above.
    PROC_SELF_FD
        .get_or_try_init(|| {
            let (_, proc_stat) = proc()?;

            let (proc_self, proc_self_stat) = proc_self()?;

            // Open "/proc/self/fd".
            let proc_self_fd = proc_opendirat(&proc_self, zstr!("fd"))?;
            let proc_self_fd_stat = check_proc_entry(
                Kind::Fd,
                proc_self_fd.as_fd(),
                Some(proc_stat),
                proc_self_stat.st_uid,
                proc_self_stat.st_gid,
            )
            .map_err(|_err| io::Error::NOTSUP)?;

            Ok(new_static_fd(proc_self_fd, proc_self_fd_stat))
        })
        .map(|(owned, _stat)| owned.as_fd())
}

type StaticFd = OnceCell<(OwnedFd, Stat)>;

#[inline]
fn new_static_fd(fd: OwnedFd, stat: Stat) -> (OwnedFd, Stat) {
    (fd, stat)
}

/// Returns a handle to Linux's `/proc/self/fdinfo` directory.
///
/// This ensures that `/proc/self/fdinfo` is `procfs`, that nothing is mounted
/// on top of it, and that it looks normal. It also returns the `Stat` of
/// `/proc/self/fd`.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man5/proc.5.html
fn proc_self_fdinfo() -> io::Result<(BorrowedFd<'static>, &'static Stat)> {
    static PROC_SELF_FDINFO: OnceCell<(OwnedFd, Stat)> = OnceCell::new();

    PROC_SELF_FDINFO
        .get_or_try_init(|| {
            let (_, proc_stat) = proc()?;

            let (proc_self, proc_self_stat) = proc_self()?;

            // Open "/proc/self/fdinfo".
            let proc_self_fdinfo = proc_opendirat(&proc_self, zstr!("fdinfo"))?;
            let proc_self_fdinfo_stat = check_proc_entry(
                Kind::Fd,
                proc_self_fdinfo.as_fd(),
                Some(proc_stat),
                proc_self_stat.st_uid,
                proc_self_stat.st_gid,
            )
            .map_err(|_err| io::Error::NOTSUP)?;

            Ok((proc_self_fdinfo, proc_self_fdinfo_stat))
        })
        .map(|(owned, stat)| (owned.as_fd(), stat))
}

/// Returns a handle to a Linux `/proc/self/fdinfo/<fd>` file.
///
/// This ensures that `/proc/self/fdinfo/<fd>` is `procfs`, that nothing is
/// mounted on top of it, and that it looks normal.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man5/proc.5.html
#[inline]
pub fn proc_self_fdinfo_fd<Fd: AsFd>(fd: Fd) -> io::Result<OwnedFd> {
    _proc_self_fdinfo(fd.as_fd())
}

fn _proc_self_fdinfo(fd: BorrowedFd<'_>) -> io::Result<OwnedFd> {
    let (proc_self_fdinfo, proc_self_fdinfo_stat) = proc_self_fdinfo()?;
    let fd_str = DecInt::from_fd(&fd);
    open_and_check_file(proc_self_fdinfo, proc_self_fdinfo_stat, fd_str.as_z_str())
}

/// Returns a handle to a Linux `/proc/self/pagemap` file.
///
/// This ensures that `/proc/self/pagemap` is `procfs`, that nothing is
/// mounted on top of it, and that it looks normal.
///
/// # References
///  - [Linux]
///  - [Linux pagemap]
///
/// [Linux]: https://man7.org/linux/man-pages/man5/proc.5.html
/// [Linux pagemap]: https://www.kernel.org/doc/Documentation/vm/pagemap.txt
#[inline]
pub fn proc_self_pagemap() -> io::Result<OwnedFd> {
    proc_self_file(zstr!("pagemap"))
}

/// Returns a handle to a Linux `/proc/self/maps` file.
///
/// This ensures that `/proc/self/maps` is `procfs`, that nothing is
/// mounted on top of it, and that it looks normal.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man5/proc.5.html
#[inline]
pub fn proc_self_maps() -> io::Result<OwnedFd> {
    proc_self_file(zstr!("maps"))
}

/// Open a file under `/proc/self`.
fn proc_self_file(name: &ZStr) -> io::Result<OwnedFd> {
    let (proc_self, proc_self_stat) = proc_self()?;
    open_and_check_file(proc_self, proc_self_stat, name)
}

/// Open a procfs file within in `dir` and check it for bind mounts.
fn open_and_check_file(dir: BorrowedFd, dir_stat: &Stat, name: &ZStr) -> io::Result<OwnedFd> {
    let (_, proc_stat) = proc()?;

    let oflags =
        OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW | OFlags::NOCTTY | OFlags::NOATIME;
    let file = openat(&dir, name, oflags, Mode::empty()).map_err(|_err| io::Error::NOTSUP)?;
    let file_stat = fstat(&file)?;

    // Open a copy of the `dir` handle so that we can read from it
    // without modifying the current position of the original handle.
    let dot = openat(&dir, zstr!("."), oflags | OFlags::DIRECTORY, Mode::empty())
        .map_err(|_err| io::Error::NOTSUP)?;

    // Confirm that we got the same inode.
    let dot_stat = fstat(&dot).map_err(|_err| io::Error::NOTSUP)?;
    if (dot_stat.st_dev, dot_stat.st_ino) != (dir_stat.st_dev, dir_stat.st_ino) {
        return Err(io::Error::NOTSUP);
    }

    // `is_mountpoint` only works on directory mount points, not file mount
    // points. To detect file mount points, scan the parent directory to see
    // if we can find a regular file with an inode and name that matches the
    // file we just opened. If we can't find it, there could be a file bind
    // mount on top of the file we want.
    //
    // TODO: With Linux 5.8 we might be able to use `statx` and
    // `STATX_ATTR_MOUNT_ROOT` to detect mountpoints directly instead of doing
    // this scanning.
    let dir = Dir::from(dot).map_err(|_err| io::Error::NOTSUP)?;
    for entry in dir {
        let entry = entry.map_err(|_err| io::Error::NOTSUP)?;
        if entry.ino() == file_stat.st_ino
            && entry.file_type() == FileType::RegularFile
            && entry.file_name() == name
        {
            // Ok, we found it. Proceed to check the file handle and succeed.
            let _ = check_proc_entry_with_stat(
                Kind::File,
                file.as_fd(),
                file_stat,
                Some(proc_stat),
                dir_stat.st_uid,
                dir_stat.st_gid,
            )?;

            return Ok(file);
        }
    }

    Err(io::Error::NOTSUP)
}

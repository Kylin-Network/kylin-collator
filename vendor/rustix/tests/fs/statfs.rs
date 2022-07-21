#[cfg(any(target_os = "android", target_os = "linux"))]
#[test]
fn test_statfs_abi() {
    use rustix::fs::{FsWord, StatFs, NFS_SUPER_MAGIC, PROC_SUPER_MAGIC};

    // Ensure these all have consistent types.
    let t: StatFs = unsafe { std::mem::zeroed() };
    let _s: FsWord = t.f_type;
    let _u: FsWord = PROC_SUPER_MAGIC;
    let _v: FsWord = NFS_SUPER_MAGIC;

    // Ensure that after all the platform-specific dancing we have to do, this
    // constant comes out with the correct value.
    #[cfg(all(libc, not(target_env = "musl")))]
    {
        assert_eq!(
            i128::from(PROC_SUPER_MAGIC),
            i128::from(libc::PROC_SUPER_MAGIC)
        );
        assert_eq!(
            i128::from(NFS_SUPER_MAGIC),
            i128::from(libc::NFS_SUPER_MAGIC)
        );
    }

    #[cfg(linux_raw)]
    {
        assert_eq!(
            i128::from(PROC_SUPER_MAGIC),
            i128::from(linux_raw_sys::general::PROC_SUPER_MAGIC)
        );
        assert_eq!(
            i128::from(NFS_SUPER_MAGIC),
            i128::from(linux_raw_sys::general::NFS_SUPER_MAGIC)
        );
    }

    assert_eq!(PROC_SUPER_MAGIC, 0x0000_9fa0);
    assert_eq!(NFS_SUPER_MAGIC, 0x0000_6969);
}

#[test]
fn test_statfs() {
    let statfs = rustix::fs::statfs("Cargo.toml").unwrap();
    let f_blocks = statfs.f_blocks;
    assert_ne!(f_blocks, 0);
    // Previously we checked f_files != 0 here, but at least btrfs doesn't set
    // that.
}

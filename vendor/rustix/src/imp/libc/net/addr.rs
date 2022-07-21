//! IPv4, IPv6, and Socket addresses.

use super::super::c;
#[cfg(unix)]
use super::offsetof_sun_path;
#[cfg(unix)]
use crate::ffi::ZStr;
#[cfg(unix)]
use crate::io;
#[cfg(unix)]
use crate::path;
#[cfg(not(windows))]
use core::convert::TryInto;
#[cfg(unix)]
use core::fmt;
#[cfg(unix)]
use core::mem::transmute;

/// `struct sockaddr_un`
#[cfg(unix)]
#[derive(Clone)]
#[doc(alias = "sockaddr_un")]
pub struct SocketAddrUnix {
    pub(crate) unix: libc::sockaddr_un,
    #[cfg(not(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd"
    )))]
    len: libc::socklen_t,
}

#[cfg(unix)]
impl SocketAddrUnix {
    /// Construct a new Unix-domain address from a filesystem path.
    #[inline]
    pub fn new<P: path::Arg>(path: P) -> io::Result<Self> {
        path.into_with_z_str(|path| Self::_new(path))
    }

    #[inline]
    fn _new(path: &ZStr) -> io::Result<Self> {
        let mut unix = Self::init();
        let bytes = path.to_bytes_with_nul();
        if bytes.len() > unix.sun_path.len() {
            return Err(io::Error::NAMETOOLONG);
        }
        for (i, b) in bytes.iter().enumerate() {
            unix.sun_path[i] = *b as c::c_char;
        }

        #[cfg(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
            unix.sun_len = (offsetof_sun_path() + bytes.len()).try_into().unwrap();
        }

        Ok(Self {
            unix,
            #[cfg(not(any(
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "ios",
                target_os = "macos",
                target_os = "netbsd",
                target_os = "openbsd"
            )))]
            len: (offsetof_sun_path() + bytes.len()).try_into().unwrap(),
        })
    }

    /// Construct a new abstract Unix-domain address from a byte slice.
    #[cfg(any(target_os = "android", target_os = "linux"))]
    #[inline]
    pub fn new_abstract_name(name: &[u8]) -> io::Result<Self> {
        let mut unix = Self::init();
        if 1 + name.len() > unix.sun_path.len() {
            return Err(io::Error::NAMETOOLONG);
        }
        unix.sun_path[0] = b'\0' as c::c_char;
        for (i, b) in name.iter().enumerate() {
            unix.sun_path[1 + i] = *b as c::c_char;
        }
        let len = offsetof_sun_path() + 1 + name.len();
        let len = len.try_into().unwrap();
        Ok(Self {
            unix,
            #[cfg(not(any(
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "ios",
                target_os = "macos",
                target_os = "netbsd",
                target_os = "openbsd"
            )))]
            len,
        })
    }

    fn init() -> c::sockaddr_un {
        c::sockaddr_un {
            #[cfg(any(
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "ios",
                target_os = "macos",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            sun_len: 0,
            sun_family: c::AF_UNIX as _,
            #[cfg(any(
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "ios",
                target_os = "macos",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            sun_path: [0; 104],
            #[cfg(not(any(
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "ios",
                target_os = "macos",
                target_os = "netbsd",
                target_os = "openbsd"
            )))]
            sun_path: [0; 108],
        }
    }

    /// For a filesystem path address, return the path.
    #[inline]
    pub fn path(&self) -> Option<&ZStr> {
        let len = self.len();
        if len != 0 && self.unix.sun_path[0] != b'\0' as c::c_char {
            let end = len as usize - offsetof_sun_path();
            // Safety: Transmuting between `&[c_char]` and `&[u8]`.
            unsafe {
                Some(ZStr::from_bytes_with_nul(transmute(&self.unix.sun_path[..end])).unwrap())
            }
        } else {
            None
        }
    }

    /// For an abstract address, return the identifier.
    #[inline]
    #[cfg(any(target_os = "android", target_os = "linux"))]
    pub fn abstract_name(&self) -> Option<&[u8]> {
        let len = self.len();
        if len != 0 && self.unix.sun_path[0] == b'\0' as c::c_char {
            let end = len as usize - offsetof_sun_path();
            // Safety: Transmuting between `&[c_char]` and `&[u8]`.
            unsafe { Some(transmute(&self.unix.sun_path[1..end])) }
        } else {
            None
        }
    }

    #[inline]
    pub(crate) fn addr_len(&self) -> c::socklen_t {
        #[cfg(not(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd"
        )))]
        {
            self.len
        }
        #[cfg(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
            c::socklen_t::from(self.unix.sun_len)
        }
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.addr_len() as usize
    }
}

#[cfg(unix)]
impl PartialEq for SocketAddrUnix {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let self_len = self.len() - offsetof_sun_path();
        let other_len = other.len() - offsetof_sun_path();
        self.unix.sun_path[..self_len].eq(&other.unix.sun_path[..other_len])
    }
}

#[cfg(unix)]
impl Eq for SocketAddrUnix {}

#[cfg(unix)]
impl PartialOrd for SocketAddrUnix {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        let self_len = self.len() - offsetof_sun_path();
        let other_len = other.len() - offsetof_sun_path();
        self.unix.sun_path[..self_len].partial_cmp(&other.unix.sun_path[..other_len])
    }
}

#[cfg(unix)]
impl Ord for SocketAddrUnix {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let self_len = self.len() - offsetof_sun_path();
        let other_len = other.len() - offsetof_sun_path();
        self.unix.sun_path[..self_len].cmp(&other.unix.sun_path[..other_len])
    }
}

#[cfg(unix)]
impl core::hash::Hash for SocketAddrUnix {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        let self_len = self.len() - offsetof_sun_path();
        self.unix.sun_path[..self_len].hash(state)
    }
}

#[cfg(unix)]
impl fmt::Debug for SocketAddrUnix {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(path) = self.path() {
            path.fmt(fmt)
        } else {
            #[cfg(any(target_os = "android", target_os = "linux"))]
            if let Some(name) = self.abstract_name() {
                return name.fmt(fmt);
            }

            "(unnamed)".fmt(fmt)
        }
    }
}

/// `struct sockaddr_storage` as a raw struct.
pub type SocketAddrStorage = c::sockaddr_storage;

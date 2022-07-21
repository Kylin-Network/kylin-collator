//! IPv4, IPv6, and Socket addresses.
//!
//! # Safety
//!
//! Linux's IPv6 type contains a union.
#![allow(unsafe_code)]

use super::super::c;
use super::offsetof_sun_path;
use crate::ffi::ZStr;
use crate::{io, path};
use core::convert::TryInto;
use core::fmt;
use core::mem::transmute;

/// `struct sockaddr_un`
#[derive(Clone)]
#[doc(alias = "sockaddr_un")]
pub struct SocketAddrUnix {
    pub(crate) unix: linux_raw_sys::general::sockaddr_un,
    len: linux_raw_sys::general::socklen_t,
}

impl SocketAddrUnix {
    /// Construct a new Unix-domain address from a filesystem path.
    #[inline]
    pub fn new<P: path::Arg>(path: P) -> io::Result<Self> {
        path.into_with_z_str(Self::_new)
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
        let len = offsetof_sun_path() + bytes.len();
        let len = len.try_into().unwrap();
        Ok(Self { unix, len })
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
        Ok(Self { unix, len })
    }

    fn init() -> linux_raw_sys::general::sockaddr_un {
        linux_raw_sys::general::sockaddr_un {
            sun_family: linux_raw_sys::general::AF_UNIX as _,
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
    pub(crate) fn addr_len(&self) -> linux_raw_sys::general::socklen_t {
        self.len
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.addr_len() as usize
    }
}

impl PartialEq for SocketAddrUnix {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let self_len = self.len() - offsetof_sun_path();
        let other_len = other.len() - offsetof_sun_path();
        self.unix.sun_path[..self_len].eq(&other.unix.sun_path[..other_len])
    }
}

impl Eq for SocketAddrUnix {}

impl PartialOrd for SocketAddrUnix {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        let self_len = self.len() - offsetof_sun_path();
        let other_len = other.len() - offsetof_sun_path();
        self.unix.sun_path[..self_len].partial_cmp(&other.unix.sun_path[..other_len])
    }
}

impl Ord for SocketAddrUnix {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let self_len = self.len() - offsetof_sun_path();
        let other_len = other.len() - offsetof_sun_path();
        self.unix.sun_path[..self_len].cmp(&other.unix.sun_path[..other_len])
    }
}

impl core::hash::Hash for SocketAddrUnix {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        let self_len = self.len() - offsetof_sun_path();
        self.unix.sun_path[..self_len].hash(state)
    }
}

impl fmt::Debug for SocketAddrUnix {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(path) = self.path() {
            path.fmt(fmt)
        } else if let Some(name) = self.abstract_name() {
            name.fmt(fmt)
        } else {
            "(unnamed)".fmt(fmt)
        }
    }
}

/// `struct sockaddr_storage` as a raw struct.
pub type SocketAddrStorage = linux_raw_sys::general::sockaddr;

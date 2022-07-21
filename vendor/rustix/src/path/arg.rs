//! This module defines the `Arg` trait and implements it for several common
//! string types.

use crate::ffi::{ZStr, ZString};
use crate::io;
#[cfg(feature = "itoa")]
use crate::path::DecInt;
use crate::path::SMALL_PATH_BUFFER_SIZE;
use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use core::str;
#[cfg(feature = "std")]
use std::ffi::{OsStr, OsString};
#[cfg(feature = "std")]
#[cfg(target_os = "hermit")]
use std::os::hermit::ext::ffi::{OsStrExt, OsStringExt};
#[cfg(feature = "std")]
#[cfg(unix)]
use std::os::unix::ffi::{OsStrExt, OsStringExt};
#[cfg(feature = "std")]
#[cfg(target_os = "vxworks")]
use std::os::vxworks::ext::ffi::{OsStrExt, OsStringExt};
#[cfg(feature = "std")]
#[cfg(target_os = "wasi")]
use std::os::wasi::ffi::{OsStrExt, OsStringExt};
#[cfg(feature = "std")]
use std::path::{Component, Components, Iter, Path, PathBuf};

/// A trait for passing path arguments.
///
/// This is similar to [`AsRef`]`<`[`Path`]`>`, but is implemented for more
/// kinds of strings and can convert into more kinds of strings.
///
/// # Example
///
/// ```rust
/// use rustix::ffi::ZStr;
/// use rustix::io;
/// use rustix::path::Arg;
///
/// pub fn touch<P: Arg>(path: P) -> io::Result<()> {
///     let path = path.into_z_str()?;
///     _touch(&path)
/// }
///
/// fn _touch(path: &ZStr) -> io::Result<()> {
///     // implementation goes here
///     Ok(())
/// }
/// ```
///
/// Users can then call `touch("foo")`, `touch(zstr!("foo"))`,
/// `touch(Path::new("foo"))`, or many other things.
///
/// [`AsRef`]: std::convert::AsRef
pub trait Arg {
    /// Returns a view of this string as a string slice.
    fn as_str(&self) -> io::Result<&str>;

    /// Returns a potentially-lossy rendering of this string as a `Cow<'_,
    /// str>`.
    fn to_string_lossy(&self) -> Cow<'_, str>;

    /// Returns a view of this string as a maybe-owned [`ZStr`].
    fn as_cow_z_str(&self) -> io::Result<Cow<'_, ZStr>>;

    /// Consumes `self` and returns a view of this string as a maybe-owned
    /// [`ZStr`].
    fn into_z_str<'b>(self) -> io::Result<Cow<'b, ZStr>>
    where
        Self: 'b;

    /// Runs a closure with `self` passed in as a `&ZStr`.
    fn into_with_z_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&ZStr) -> io::Result<T>;

    /// Returns a view of this string as a maybe-owned [`ZStr`].
    #[cfg(not(feature = "rustc-dep-of-std"))]
    #[inline]
    fn as_cow_c_str(&self) -> io::Result<Cow<'_, ZStr>> {
        self.as_cow_z_str()
    }

    /// Consumes `self` and returns a view of this string as a maybe-owned
    /// [`ZStr`].
    #[cfg(not(feature = "rustc-dep-of-std"))]
    #[inline]
    fn into_c_str<'b>(self) -> io::Result<Cow<'b, ZStr>>
    where
        Self: 'b + Sized,
    {
        self.into_z_str()
    }

    /// Runs a closure with `self` passed in as a `&ZStr`.
    #[cfg(not(feature = "rustc-dep-of-std"))]
    #[inline]
    fn into_with_c_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&ZStr) -> io::Result<T>,
    {
        self.into_with_z_str(f)
    }
}

impl Arg for &str {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        Ok(self)
    }

    #[inline]
    fn to_string_lossy(&self) -> Cow<'_, str> {
        Cow::Borrowed(self)
    }

    #[inline]
    fn as_cow_z_str(&self) -> io::Result<Cow<'_, ZStr>> {
        Ok(Cow::Owned(
            ZString::new(self.as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_z_str<'b>(self) -> io::Result<Cow<'b, ZStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            ZString::new(self).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_with_z_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&ZStr) -> io::Result<T>,
    {
        with_z_str(self.as_bytes(), f)
    }
}

impl Arg for &String {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        Ok(self)
    }

    #[inline]
    fn to_string_lossy(&self) -> Cow<'_, str> {
        Cow::Borrowed(self)
    }

    #[inline]
    fn as_cow_z_str(&self) -> io::Result<Cow<'_, ZStr>> {
        Ok(Cow::Owned(
            ZString::new(String::as_str(self).as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_z_str<'b>(self) -> io::Result<Cow<'b, ZStr>>
    where
        Self: 'b,
    {
        self.as_str().into_z_str()
    }

    #[inline]
    fn into_with_z_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&ZStr) -> io::Result<T>,
    {
        with_z_str(self.as_bytes(), f)
    }
}

impl Arg for String {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        Ok(self)
    }

    #[inline]
    fn to_string_lossy(&self) -> Cow<'_, str> {
        Cow::Borrowed(self)
    }

    #[inline]
    fn as_cow_z_str(&self) -> io::Result<Cow<'_, ZStr>> {
        Ok(Cow::Owned(
            ZString::new(self.as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_z_str<'b>(self) -> io::Result<Cow<'b, ZStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            ZString::new(self).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_with_z_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&ZStr) -> io::Result<T>,
    {
        f(&ZString::new(self).map_err(|_cstr_err| io::Error::INVAL)?)
    }
}

#[cfg(feature = "std")]
impl Arg for &OsStr {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        self.to_str().ok_or(io::Error::INVAL)
    }

    #[inline]
    fn to_string_lossy(&self) -> Cow<'_, str> {
        OsStr::to_string_lossy(self)
    }

    #[inline]
    fn as_cow_z_str(&self) -> io::Result<Cow<'_, ZStr>> {
        Ok(Cow::Owned(
            ZString::new(self.as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_z_str<'b>(self) -> io::Result<Cow<'b, ZStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            ZString::new(self.as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_with_z_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&ZStr) -> io::Result<T>,
    {
        with_z_str(self.as_bytes(), f)
    }
}

#[cfg(feature = "std")]
impl Arg for &OsString {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        OsString::as_os_str(self).to_str().ok_or(io::Error::INVAL)
    }

    #[inline]
    fn to_string_lossy(&self) -> Cow<'_, str> {
        self.as_os_str().to_string_lossy()
    }

    #[inline]
    fn as_cow_z_str(&self) -> io::Result<Cow<'_, ZStr>> {
        Ok(Cow::Owned(
            ZString::new(OsString::as_os_str(self).as_bytes())
                .map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_z_str<'b>(self) -> io::Result<Cow<'b, ZStr>>
    where
        Self: 'b,
    {
        self.as_os_str().into_z_str()
    }

    #[inline]
    fn into_with_z_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&ZStr) -> io::Result<T>,
    {
        with_z_str(self.as_bytes(), f)
    }
}

#[cfg(feature = "std")]
impl Arg for OsString {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        self.as_os_str().to_str().ok_or(io::Error::INVAL)
    }

    #[inline]
    fn to_string_lossy(&self) -> Cow<'_, str> {
        self.as_os_str().to_string_lossy()
    }

    #[inline]
    fn as_cow_z_str(&self) -> io::Result<Cow<'_, ZStr>> {
        Ok(Cow::Owned(
            ZString::new(self.as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_z_str<'b>(self) -> io::Result<Cow<'b, ZStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            ZString::new(self.into_vec()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_with_z_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&ZStr) -> io::Result<T>,
    {
        f(&ZString::new(self.into_vec()).map_err(|_cstr_err| io::Error::INVAL)?)
    }
}

#[cfg(feature = "std")]
impl Arg for &Path {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        self.as_os_str().to_str().ok_or(io::Error::INVAL)
    }

    #[inline]
    fn to_string_lossy(&self) -> Cow<'_, str> {
        Path::to_string_lossy(self)
    }

    #[inline]
    fn as_cow_z_str(&self) -> io::Result<Cow<'_, ZStr>> {
        Ok(Cow::Owned(
            ZString::new(self.as_os_str().as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_z_str<'b>(self) -> io::Result<Cow<'b, ZStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            ZString::new(self.as_os_str().as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_with_z_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&ZStr) -> io::Result<T>,
    {
        with_z_str(self.as_os_str().as_bytes(), f)
    }
}

#[cfg(feature = "std")]
impl Arg for &PathBuf {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        PathBuf::as_path(self)
            .as_os_str()
            .to_str()
            .ok_or(io::Error::INVAL)
    }

    #[inline]
    fn to_string_lossy(&self) -> Cow<'_, str> {
        self.as_path().to_string_lossy()
    }

    #[inline]
    fn as_cow_z_str(&self) -> io::Result<Cow<'_, ZStr>> {
        Ok(Cow::Owned(
            ZString::new(PathBuf::as_path(self).as_os_str().as_bytes())
                .map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_z_str<'b>(self) -> io::Result<Cow<'b, ZStr>>
    where
        Self: 'b,
    {
        self.as_path().into_z_str()
    }

    #[inline]
    fn into_with_z_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&ZStr) -> io::Result<T>,
    {
        with_z_str(self.as_os_str().as_bytes(), f)
    }
}

#[cfg(feature = "std")]
impl Arg for PathBuf {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        self.as_os_str().to_str().ok_or(io::Error::INVAL)
    }

    #[inline]
    fn to_string_lossy(&self) -> Cow<'_, str> {
        self.as_os_str().to_string_lossy()
    }

    #[inline]
    fn as_cow_z_str(&self) -> io::Result<Cow<'_, ZStr>> {
        Ok(Cow::Owned(
            ZString::new(self.as_os_str().as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_z_str<'b>(self) -> io::Result<Cow<'b, ZStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            ZString::new(self.into_os_string().into_vec()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_with_z_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&ZStr) -> io::Result<T>,
    {
        f(
            &ZString::new(self.into_os_string().into_vec())
                .map_err(|_cstr_err| io::Error::INVAL)?,
        )
    }
}

impl Arg for &ZStr {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        self.to_str().map_err(|_utf8_err| io::Error::INVAL)
    }

    #[inline]
    fn to_string_lossy(&self) -> Cow<'_, str> {
        ZStr::to_string_lossy(self)
    }

    #[inline]
    fn as_cow_z_str(&self) -> io::Result<Cow<'_, ZStr>> {
        Ok(Cow::Borrowed(self))
    }

    #[inline]
    fn into_z_str<'b>(self) -> io::Result<Cow<'b, ZStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Borrowed(self))
    }

    #[inline]
    fn into_with_z_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&ZStr) -> io::Result<T>,
    {
        f(self)
    }
}

impl Arg for &ZString {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        unimplemented!()
    }

    #[inline]
    fn to_string_lossy(&self) -> Cow<'_, str> {
        unimplemented!()
    }

    #[inline]
    fn as_cow_z_str(&self) -> io::Result<Cow<'_, ZStr>> {
        Ok(Cow::Borrowed(self))
    }

    #[inline]
    fn into_z_str<'b>(self) -> io::Result<Cow<'b, ZStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Borrowed(self))
    }

    #[inline]
    fn into_with_z_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&ZStr) -> io::Result<T>,
    {
        f(self)
    }
}

impl Arg for ZString {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        self.to_str().map_err(|_utf8_err| io::Error::INVAL)
    }

    #[inline]
    fn to_string_lossy(&self) -> Cow<'_, str> {
        ZStr::to_string_lossy(self)
    }

    #[inline]
    fn as_cow_z_str(&self) -> io::Result<Cow<'_, ZStr>> {
        Ok(Cow::Borrowed(self))
    }

    #[inline]
    fn into_z_str<'b>(self) -> io::Result<Cow<'b, ZStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(self))
    }

    #[inline]
    fn into_with_z_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&ZStr) -> io::Result<T>,
    {
        f(&self)
    }
}

impl<'a> Arg for Cow<'a, str> {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        Ok(self)
    }

    #[inline]
    fn to_string_lossy(&self) -> Cow<'_, str> {
        Cow::Borrowed(self)
    }

    #[inline]
    fn as_cow_z_str(&self) -> io::Result<Cow<'_, ZStr>> {
        Ok(Cow::Owned(
            ZString::new(self.as_ref()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_z_str<'b>(self) -> io::Result<Cow<'b, ZStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            match self {
                Cow::Owned(s) => ZString::new(s),
                Cow::Borrowed(s) => ZString::new(s),
            }
            .map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_with_z_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&ZStr) -> io::Result<T>,
    {
        with_z_str(self.as_bytes(), f)
    }
}

#[cfg(feature = "std")]
impl<'a> Arg for Cow<'a, OsStr> {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        (**self).to_str().ok_or(io::Error::INVAL)
    }

    #[inline]
    fn to_string_lossy(&self) -> Cow<'_, str> {
        (**self).to_string_lossy()
    }

    #[inline]
    fn as_cow_z_str(&self) -> io::Result<Cow<'_, ZStr>> {
        Ok(Cow::Owned(
            ZString::new(self.as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_z_str<'b>(self) -> io::Result<Cow<'b, ZStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            match self {
                Cow::Owned(os) => ZString::new(os.into_vec()),
                Cow::Borrowed(os) => ZString::new(os.as_bytes()),
            }
            .map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_with_z_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&ZStr) -> io::Result<T>,
    {
        with_z_str(self.as_bytes(), f)
    }
}

impl<'a> Arg for Cow<'a, ZStr> {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        self.to_str().map_err(|_utf8_err| io::Error::INVAL)
    }

    #[inline]
    fn to_string_lossy(&self) -> Cow<'_, str> {
        let borrow: &ZStr = core::borrow::Borrow::borrow(self);
        borrow.to_string_lossy()
    }

    #[inline]
    fn as_cow_z_str(&self) -> io::Result<Cow<'_, ZStr>> {
        Ok(Cow::Borrowed(self))
    }

    #[inline]
    fn into_z_str<'b>(self) -> io::Result<Cow<'b, ZStr>>
    where
        Self: 'b,
    {
        Ok(self)
    }

    #[inline]
    fn into_with_z_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&ZStr) -> io::Result<T>,
    {
        f(&self)
    }
}

#[cfg(feature = "std")]
impl<'a> Arg for Component<'a> {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        self.as_os_str().to_str().ok_or(io::Error::INVAL)
    }

    #[inline]
    fn to_string_lossy(&self) -> Cow<'_, str> {
        self.as_os_str().to_string_lossy()
    }

    #[inline]
    fn as_cow_z_str(&self) -> io::Result<Cow<'_, ZStr>> {
        Ok(Cow::Owned(
            ZString::new(self.as_os_str().as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_z_str<'b>(self) -> io::Result<Cow<'b, ZStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            ZString::new(self.as_os_str().as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_with_z_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&ZStr) -> io::Result<T>,
    {
        with_z_str(self.as_os_str().as_bytes(), f)
    }
}

#[cfg(feature = "std")]
impl<'a> Arg for Components<'a> {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        self.as_path().to_str().ok_or(io::Error::INVAL)
    }

    #[inline]
    fn to_string_lossy(&self) -> Cow<'_, str> {
        self.as_path().to_string_lossy()
    }

    #[inline]
    fn as_cow_z_str(&self) -> io::Result<Cow<'_, ZStr>> {
        Ok(Cow::Owned(
            ZString::new(self.as_path().as_os_str().as_bytes())
                .map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_z_str<'b>(self) -> io::Result<Cow<'b, ZStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            ZString::new(self.as_path().as_os_str().as_bytes())
                .map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_with_z_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&ZStr) -> io::Result<T>,
    {
        with_z_str(self.as_path().as_os_str().as_bytes(), f)
    }
}

#[cfg(feature = "std")]
impl<'a> Arg for Iter<'a> {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        self.as_path().to_str().ok_or(io::Error::INVAL)
    }

    #[inline]
    fn to_string_lossy(&self) -> Cow<'_, str> {
        self.as_path().to_string_lossy()
    }

    #[inline]
    fn as_cow_z_str(&self) -> io::Result<Cow<'_, ZStr>> {
        Ok(Cow::Owned(
            ZString::new(self.as_path().as_os_str().as_bytes())
                .map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_z_str<'b>(self) -> io::Result<Cow<'b, ZStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            ZString::new(self.as_path().as_os_str().as_bytes())
                .map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_with_z_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&ZStr) -> io::Result<T>,
    {
        with_z_str(self.as_path().as_os_str().as_bytes(), f)
    }
}

impl Arg for &[u8] {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        str::from_utf8(self).map_err(|_utf8_err| io::Error::INVAL)
    }

    #[inline]
    fn to_string_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(self)
    }

    #[inline]
    fn as_cow_z_str(&self) -> io::Result<Cow<'_, ZStr>> {
        Ok(Cow::Owned(
            ZString::new(*self).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_z_str<'b>(self) -> io::Result<Cow<'b, ZStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            ZString::new(self).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_with_z_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&ZStr) -> io::Result<T>,
    {
        with_z_str(self, f)
    }
}

impl Arg for &Vec<u8> {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        str::from_utf8(self).map_err(|_utf8_err| io::Error::INVAL)
    }

    #[inline]
    fn to_string_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(self)
    }

    #[inline]
    fn as_cow_z_str(&self) -> io::Result<Cow<'_, ZStr>> {
        Ok(Cow::Owned(
            ZString::new(self.as_slice()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_z_str<'b>(self) -> io::Result<Cow<'b, ZStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            ZString::new(self.as_slice()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_with_z_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&ZStr) -> io::Result<T>,
    {
        with_z_str(self, f)
    }
}

impl Arg for Vec<u8> {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        str::from_utf8(self).map_err(|_utf8_err| io::Error::INVAL)
    }

    #[inline]
    fn to_string_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(self)
    }

    #[inline]
    fn as_cow_z_str(&self) -> io::Result<Cow<'_, ZStr>> {
        Ok(Cow::Owned(
            ZString::new(self.as_slice()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_z_str<'b>(self) -> io::Result<Cow<'b, ZStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            ZString::new(self).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_with_z_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&ZStr) -> io::Result<T>,
    {
        f(&ZString::new(self).map_err(|_cstr_err| io::Error::INVAL)?)
    }
}

#[cfg(feature = "itoa")]
impl Arg for DecInt {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        Ok(self.as_str())
    }

    #[inline]
    fn to_string_lossy(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.as_str())
    }

    #[inline]
    fn as_cow_z_str(&self) -> io::Result<Cow<'_, ZStr>> {
        Ok(Cow::Owned(
            ZString::new(self.as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_z_str<'b>(self) -> io::Result<Cow<'b, ZStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            ZString::new(self.as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_with_z_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&ZStr) -> io::Result<T>,
    {
        f(self.as_z_str())
    }
}

/// Runs a closure with `bytes` passed in as a `&ZStr`.
#[inline]
fn with_z_str<T, F>(bytes: &[u8], f: F) -> io::Result<T>
where
    F: FnOnce(&ZStr) -> io::Result<T>,
{
    // Most paths are less than `SMALL_PATH_BUFFER_SIZE` long. The rest can go
    // through the dynamic allocation path. If you're opening many files in a
    // directory with a long path, consider opening the directory and using
    // `openat` to open the files under it, which will avoid this, and is often
    // faster in the OS as well.

    // Test with >= so that we have room for the trailing NUL.
    if bytes.len() >= SMALL_PATH_BUFFER_SIZE {
        return with_z_str_slow_path(bytes, f);
    }
    let mut buffer: [u8; SMALL_PATH_BUFFER_SIZE] = [0_u8; SMALL_PATH_BUFFER_SIZE];
    // Copy the bytes in; the buffer already has zeros for the trailing NUL.
    buffer[..bytes.len()].copy_from_slice(bytes);
    f(ZStr::from_bytes_with_nul(&buffer[..=bytes.len()]).map_err(|_cstr_err| io::Error::INVAL)?)
}

/// The slow path which handles any length. In theory OS's only support up
/// to `PATH_MAX`, but we let the OS enforce that.
#[cold]
fn with_z_str_slow_path<T, F>(bytes: &[u8], f: F) -> io::Result<T>
where
    F: FnOnce(&ZStr) -> io::Result<T>,
{
    f(&ZString::new(bytes).map_err(|_cstr_err| io::Error::INVAL)?)
}

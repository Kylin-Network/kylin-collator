extern crate libc;
extern crate libloading;

use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::{io, ffi};
use self::libloading::os::unix::{Library, Symbol};

lazy_static! {
	/// `renamex_np` is available only on macos >= 10.12
	static ref RENAMEX_NP: Option<Symbol<unsafe extern fn (oldpath: *const libc::c_char, newpath: *const libc::c_char, flags: libc::c_uint) -> libc::c_int>> = unsafe {
		let lib = Library::this();
		lib.get(b"renamex_np").ok()
	};
}

#[cfg(target_arch = "x86_64")]
extern "stdcall" {
	fn exchangedata(oldpath: *const libc::c_char, newpath: *const libc::c_char, flags: libc::c_uint) -> libc::c_int;
}

pub fn swap<A, B>(a: A, b: B) -> io::Result<()> where A: AsRef<Path>, B: AsRef<Path> {
	const RENAME_SWAP: libc::c_uint = 2;

	let a_path = ffi::CString::new(a.as_ref().as_os_str().as_bytes())?;
	let b_path = ffi::CString::new(b.as_ref().as_os_str().as_bytes())?;

	unsafe {
		// `renamex_np` is available only on macos >= 10.12
		// if not available, let's fallback to `exchangedata`
		if let Some(ref renamex_np) = &*RENAMEX_NP {
			if renamex_np(a_path.as_ptr(), b_path.as_ptr(), RENAME_SWAP) == 0 {
				return Ok(())
			}

			// `renamex_np` throws ENOTSUP if volume is not APFS
			// if it's not APFS, let's fallback to `exchangedata`
			let err = *libc::__error();
			if err != libc::ENOTSUP {
				return Err(io::Error::new(io::ErrorKind::Other, format!("renamex_np failed with code: {}", err)));
			}
		}

		#[cfg(target_arch = "x86_64")]
		// `exchangedata` does not support swapping directories
		if exchangedata(a_path.as_ptr(), b_path.as_ptr(), 0) == 0 {
			return Ok(())
		}

		Err(io::Error::new(io::ErrorKind::Other, format!("exchangedata failed with code: {}", *libc::__error())))
	}
}

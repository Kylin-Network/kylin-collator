extern crate winapi;

use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::{OsStringExt, OsStrExt};
use std::path::Path;
use std::{io, fs, ptr};

use self::winapi::shared::minwindef::{MAX_PATH, FALSE};
use self::winapi::shared::ntdef::HANDLE;
use self::winapi::um::errhandlingapi::GetLastError;
use self::winapi::um::fileapi::GetTempFileNameW;
use self::winapi::um::handleapi::INVALID_HANDLE_VALUE;
use self::winapi::um::ktmw32::{CreateTransaction, RollbackTransaction, CommitTransaction};
use self::winapi::um::winbase::{MoveFileTransactedW, DeleteFileTransactedW};

/// Function used to create a temporary file in given directory
fn tmp_name_in_dir(dir: &Path) -> io::Result<OsString> {
	let mut out = Vec::with_capacity(MAX_PATH);
	let pre: Vec<u16> = OsStr::new("tmp").encode_wide().chain(Some(0)).collect();
	let dir: Vec<u16> = OsStr::new(dir).encode_wide().chain(Some(0)).collect();

	unsafe {
		if GetTempFileNameW(dir.as_ptr(), pre.as_ptr(), 0, out.as_mut_ptr()) != 0 {
			out.set_len(MAX_PATH);
			let n = out.iter()
				.position(|&x| x == 0)
				.ok_or_else(|| io::Error::new(io::ErrorKind::Other, "GetTempFileNameW returned invalid path".to_string()))?;
			Ok(OsString::from_wide(&out[..n]))
		} else {
			Err(io::Error::new(io::ErrorKind::Other, format!("GetTempFileNameW failed with code: {}", GetLastError())))
		}
	}
}

struct Transaction(HANDLE);

impl Transaction {
	fn new() -> io::Result<Self> {
		unsafe {
			let handle = CreateTransaction(ptr::null_mut(), ptr::null_mut(), 0, 0, 0, 0, ptr::null_mut());
			if handle == INVALID_HANDLE_VALUE {
				return Err(io::Error::new(io::ErrorKind::Other, format!("ktmw32::CreateTransaction failed with code: {}", GetLastError())));
			}
			Ok(Transaction(handle))
		}
	}

	fn delete_file<A>(&self, a: A) -> io::Result<()> where A: AsRef<OsStr> {
		let a: Vec<u16> = a.as_ref().encode_wide().chain(Some(0)).collect();

		unsafe {
			if DeleteFileTransactedW(a.as_ptr(), self.0) != FALSE {
				Ok(())
			} else {
				let error = GetLastError();
				self.rollback();
				Err(io::Error::new(io::ErrorKind::Other, format!("DeleteFileTransactedW failed with code: {}", error)))
			}
		}
	}

	fn move_file<A, B>(&self, a: A, b: B) -> io::Result<()> where A: AsRef<OsStr>, B: AsRef<OsStr> {
		let a: Vec<u16> = a.as_ref().encode_wide().chain(Some(0)).collect();
		let b: Vec<u16> = b.as_ref().encode_wide().chain(Some(0)).collect();

		unsafe {
			if MoveFileTransactedW(a.as_ptr(), b.as_ptr(), None, ptr::null_mut(), 0, self.0) != FALSE {
				Ok(())
			} else {
				let error = GetLastError();
				self.rollback();
				Err(io::Error::new(io::ErrorKind::Other, format!("MoveFileTransactedW failed with code: {}", error)))
			}
		}
	}

	fn commit(&self) -> io::Result<()> {
		unsafe {
			if CommitTransaction(self.0) != FALSE {
				Ok(())
			} else {
				let error = GetLastError();
				self.rollback();
				Err(io::Error::new(io::ErrorKind::Other, format!("CommitTransaction failed with code: {}", error)))
			}
		}
	}

	fn rollback(&self) {
		unsafe {
			// we simply ignore an error over here, cause there is nothing we can do to recover from this state
			let _ = RollbackTransaction(self.0);
		}
	}
}

pub fn swap<A, B>(a: A, b: B) -> io::Result<()> where A: AsRef<Path>, B: AsRef<Path> {
	let a = fs::canonicalize(a)?;
	let b = fs::canonicalize(b)?;

	let parent_dir = a.parent()
		.or_else(|| b.parent())
		.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Could not find a parent directory"))?;

	let tmp = tmp_name_in_dir(parent_dir)?;

	let transaction = Transaction::new()?;
	transaction.delete_file(&tmp)?;
	transaction.move_file(&a, &tmp)?;
	transaction.move_file(&b, &a)?;
	transaction.move_file(&tmp, &b)?;
	transaction.commit()
}

use std::io;
use std::path::Path;

pub fn swap<A, B>(_a: A, _b: B) -> io::Result<()> where A: AsRef<Path>, B: AsRef<Path> {
	Err(io::Error::new(io::ErrorKind::Other, "PathSwap not supported by the current platform"))
}

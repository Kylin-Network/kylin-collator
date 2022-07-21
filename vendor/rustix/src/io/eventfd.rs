use crate::imp;
use crate::io::{self, OwnedFd};

pub use imp::io::EventfdFlags;

/// `eventfd(initval, flags)`—Creates a file descriptor for event
/// notification.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/eventfd.2.html
#[inline]
pub fn eventfd(initval: u32, flags: EventfdFlags) -> io::Result<OwnedFd> {
    imp::io::syscalls::eventfd(initval, flags)
}

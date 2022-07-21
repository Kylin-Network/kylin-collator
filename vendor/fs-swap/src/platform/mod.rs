#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;
#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
mod unsupported;

#[cfg(target_os = "linux")]
pub use self::linux::swap;
#[cfg(target_os = "macos")]
pub use self::macos::swap;
#[cfg(windows)]
pub use self::windows::swap;
#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
pub use self::unsupported::swap;

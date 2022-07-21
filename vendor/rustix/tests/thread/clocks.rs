#[cfg(not(any(
    target_os = "emscripten",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "openbsd",
    target_os = "redox",
    target_os = "wasi",
)))]
use rustix::{
    thread::{clock_nanosleep_absolute, clock_nanosleep_relative},
    time::ClockId,
};
#[cfg(not(target_os = "redox"))]
use {
    rustix::io,
    rustix::thread::{nanosleep, NanosleepRelativeResult},
    rustix::time::Timespec,
};

#[cfg(not(target_os = "redox"))]
#[test]
fn test_invalid_nanosleep() {
    match nanosleep(&Timespec {
        tv_sec: 0,
        tv_nsec: 1000000000,
    }) {
        NanosleepRelativeResult::Err(io::Error::INVAL) => (),
        otherwise => panic!("unexpected resut: {:?}", otherwise),
    }
    match nanosleep(&Timespec {
        tv_sec: 0,
        tv_nsec: -1 as _,
    }) {
        NanosleepRelativeResult::Err(io::Error::INVAL) => (),
        otherwise => panic!("unexpected resut: {:?}", otherwise),
    }
}

#[cfg(not(any(
    target_os = "emscripten",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "openbsd",
    target_os = "redox",
    target_os = "wasi",
)))]
#[test]
fn test_invalid_nanosleep_absolute() {
    match clock_nanosleep_absolute(
        ClockId::Monotonic,
        &Timespec {
            tv_sec: 0,
            tv_nsec: 1000000000,
        },
    ) {
        Err(io::Error::INVAL) => (),
        otherwise => panic!("unexpected resut: {:?}", otherwise),
    }
    match clock_nanosleep_absolute(
        ClockId::Monotonic,
        &Timespec {
            tv_sec: 0,
            tv_nsec: -1 as _,
        },
    ) {
        Err(io::Error::INVAL) => (),
        otherwise => panic!("unexpected resut: {:?}", otherwise),
    }
}

#[cfg(not(any(
    target_os = "emscripten",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "openbsd",
    target_os = "redox",
    target_os = "wasi",
)))]
#[test]
fn test_invalid_nanosleep_relative() {
    match clock_nanosleep_relative(
        ClockId::Monotonic,
        &Timespec {
            tv_sec: 0,
            tv_nsec: 1000000000,
        },
    ) {
        NanosleepRelativeResult::Err(io::Error::INVAL) => (),
        otherwise => panic!("unexpected resut: {:?}", otherwise),
    }
    match clock_nanosleep_relative(
        ClockId::Monotonic,
        &Timespec {
            tv_sec: 0,
            tv_nsec: -1 as _,
        },
    ) {
        NanosleepRelativeResult::Err(io::Error::INVAL) => (),
        otherwise => panic!("unexpected resut: {:?}", otherwise),
    }
}

#[cfg(not(target_os = "redox"))]
#[test]
fn test_zero_nanosleep() {
    match nanosleep(&Timespec {
        tv_sec: 0,
        tv_nsec: 0,
    }) {
        NanosleepRelativeResult::Ok => (),
        otherwise => panic!("unexpected resut: {:?}", otherwise),
    }
}

#[cfg(not(any(
    target_os = "emscripten",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "openbsd",
    target_os = "redox",
    target_os = "wasi",
)))]
#[test]
fn test_zero_nanosleep_absolute() {
    match clock_nanosleep_absolute(
        ClockId::Monotonic,
        &Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
    ) {
        Ok(()) => (),
        otherwise => panic!("unexpected resut: {:?}", otherwise),
    }
}

#[cfg(not(any(
    target_os = "emscripten",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "openbsd",
    target_os = "redox",
    target_os = "wasi",
)))]
#[test]
fn test_zero_nanosleep_relative() {
    match clock_nanosleep_relative(
        ClockId::Monotonic,
        &Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
    ) {
        NanosleepRelativeResult::Ok => (),
        otherwise => panic!("unexpected resut: {:?}", otherwise),
    }
}

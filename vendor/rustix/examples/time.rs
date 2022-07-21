//! A command which prints the current values of the realtime and monotonic
//! clocks it's given.

#[cfg(not(windows))]
fn main() {
    println!(
        "Real time: {:?}",
        rustix::time::clock_gettime(rustix::time::ClockId::Realtime)
    );
    println!(
        "Monotonic time: {:?}",
        rustix::time::clock_gettime(rustix::time::ClockId::Monotonic)
    );
}

#[cfg(windows)]
fn main() {
    unimplemented!()
}

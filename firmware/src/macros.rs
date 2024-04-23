#[cfg(feature = "rtt")]
macro_rules! __rtt_log {
    (warn, $($arg:expr),*) => { rtt_target::rprintln!($($arg),*) };
    (debug, $($arg:expr),*) => { rtt_target::rprintln!($($arg),*) };
}

#[cfg(feature = "defmt")]
macro_rules! __rtt_log {
    (warn, $($arg:expr),*) => { defmt::warn!($($arg),*) };
    (trace, $($arg:expr),*) => { defmt::trace!($($arg),*) };
    (debug, $($arg:expr),*) => { defmt::debug!($($arg),*) };
}

#[cfg(not(any(feature = "rtt", feature = "defmt")))]
macro_rules! __rtt_log {
    ($($arg:expr),*) => {
        () // NoOp
    };
}

macro_rules! rtt_warn {
    ($($arg:expr),*) => (__rtt_log!(warn, $($arg),*));
}

macro_rules! rtt_debug {
    ($($arg:expr),*) => (__rtt_log!(debug, $($arg),*));
}

macro_rules! rtt_trace {
    ($($arg:expr),*) => (__rtt_log!(trace, $($arg),*));
}

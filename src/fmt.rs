// Copied from esp-hal: https://github.com/esp-rs/esp-hal/blob/main/esp-hal/src/fmt.rs

#![macro_use]
#![allow(unused_macros)]

macro_rules! trace {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            cfg_if::cfg_if! {
                if #[cfg(feature = "log")] {
                    ::log::trace!($s $(, $x)*);
                } else if #[cfg(feature = "defmt")] {
                    ::defmt::trace!($s $(, $x)*);
                } else {
                    let _ = ($( & $x ),*);
                }
            }
        }
    };
}

macro_rules! debug {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            cfg_if::cfg_if! {
                if #[cfg(feature = "log")] {
                    ::log::debug!($s $(, $x)*);
                } else if #[cfg(feature = "defmt")] {
                    ::defmt::debug!($s $(, $x)*);
                } else {
                    let _ = ($( & $x ),*);
                }
            }
        }
    };
}

macro_rules! info {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            cfg_if::cfg_if! {
                if #[cfg(feature = "log")] {
                    ::log::info!($s $(, $x)*);
                } else if #[cfg(feature = "defmt")] {
                    ::defmt::info!($s $(, $x)*);
                } else {
                    let _ = ($( & $x ),*);
                }
            }
        }
    };
}

macro_rules! warn {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            cfg_if::cfg_if! {
                if #[cfg(feature = "log")] {
                    ::log::warn!($s $(, $x)*);
                } else if #[cfg(feature = "defmt")] {
                    ::defmt::warn!($s $(, $x)*);
                } else {
                    let _ = ($( & $x ),*);
                }
            }
        }
    };
}

macro_rules! error {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            cfg_if::cfg_if! {
                if #[cfg(feature = "log")] {
                    ::log::error!($s $(, $x)*);
                } else if #[cfg(feature = "defmt")] {
                    ::defmt::error!($s $(, $x)*);
                } else {
                    let _ = ($( & $x ),*);
                }
            }
        }
    };
}

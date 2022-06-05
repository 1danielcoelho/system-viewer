#[derive(PartialEq, PartialOrd)]
pub enum LogLevel {
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

pub enum LogCat {
    Default,
    Engine,
    Rendering,
    Io,
    Input,
}
impl LogCat {
    pub const fn default_level(cat: &LogCat) -> LogLevel {
        match cat {
            LogCat::Default => LogLevel::Debug,
            LogCat::Engine => LogLevel::Debug,
            LogCat::Rendering => LogLevel::Error,
            LogCat::Io => LogLevel::Debug,
            LogCat::Input => LogLevel::Debug,
        }
    }

    pub const fn to_string(&self) -> &'static str {
        match self {
            LogCat::Default => "",
            LogCat::Engine => "[ENGI]: ",
            LogCat::Rendering => "[REND]: ",
            LogCat::Io => "[  IO]: ",
            LogCat::Input => "[INPT]: ",
        }
    }
}

pub fn log_internal(cat: LogCat, level: LogLevel, args: std::fmt::Arguments) {
    if level >= LogCat::default_level(&cat) {
        let log_fun = match level {
            LogLevel::Debug => web_sys::console::debug_1,
            LogLevel::Info => web_sys::console::info_1,
            LogLevel::Warn => web_sys::console::warn_1,
            LogLevel::Error => web_sys::console::error_1,
        };

        log_fun(&format!("{}{}", cat.to_string(), args.to_string()).into());
    }
}

macro_rules! debug {
    ($cat:expr, $($args:tt)+) => ({
        $crate::utils::log::log_internal(
            $cat,
            $crate::utils::log::LogLevel::Info,
            format_args!($($args)+)
        );
    });

    ($($args:tt)+) => ({
        $crate::utils::log::log_internal(
            $crate::utils::log::LogCat::Default,
            $crate::utils::log::LogLevel::Info,
            format_args!($($args)+)
        );
    });
}

macro_rules! info {
    ($cat:expr, $($args:tt)+) => ({
        $crate::utils::log::log_internal(
            $cat,
            $crate::utils::log::LogLevel::Debug,
            format_args!($($args)+)
        );
    });

    ($($args:tt)+) => ({
        $crate::utils::log::log_internal(
            $crate::utils::log::LogCat::Default,
            $crate::utils::log::LogLevel::Debug,
            format_args!($($args)+)
        );
    });
}

// macro_rules! warn {
//     ($cat:expr, $($args:tt)+) => ({
//         $crate::utils::log::log_internal(
//             $cat,
//             $crate::utils::log::LogLevel::Warn,
//             format_args!($($args)+)
//         );
//     });

//     ($($args:tt)+) => ({
//         $crate::utils::log::log_internal(
//             $crate::utils::log::LogCat::Default,
//             $crate::utils::log::LogLevel::Warn,
//             format_args!($($args)+)
//         );
//     });
// }

macro_rules! error {
    ($cat:expr, $($args:tt)+) => ({
        $crate::utils::log::log_internal(
            $cat,
            $crate::utils::log::LogLevel::Error,
            format_args!($($args)+)
        );
    });

    ($($args:tt)+) => ({
        $crate::utils::log::log_internal(
            $crate::utils::log::LogCat::Default,
            $crate::utils::log::LogLevel::Error,
            format_args!($($args)+)
        );
    });
}

pub(crate) use debug;
pub(crate) use error;
pub(crate) use info;
// pub(crate) use warn;

#[derive(PartialEq, PartialOrd)]
pub enum LogLevel {
    Debug = 1,
    Info = 2,
    Warning = 3,
    Error = 4,
}

pub enum LogCat {
    Engine,
    Scene,
    Resources,
    Physics,
    Orbit,
    Gltf,
    Io,
    Ui,
}
impl LogCat {
    pub const fn default_level(cat: &LogCat) -> LogLevel {
        match cat {
            LogCat::Engine => LogLevel::Debug,
            LogCat::Scene => LogLevel::Debug,
            LogCat::Resources => LogLevel::Debug,
            LogCat::Physics => LogLevel::Debug,
            LogCat::Orbit => LogLevel::Debug,
            LogCat::Gltf => LogLevel::Debug,
            LogCat::Io => LogLevel::Debug,
            LogCat::Ui => LogLevel::Debug,
        }
    }

    pub const fn to_string(&self) -> &'static str {
        match self {
            LogCat::Engine => "[ENGI]: ",
            LogCat::Scene => "[SCEN]: ",
            LogCat::Resources => "[RESO]: ",
            LogCat::Physics => "[PHYS]: ",
            LogCat::Orbit => "[ORBT]: ",
            LogCat::Gltf => "[GLTF]: ",
            LogCat::Io => "[  IO]: ",
            LogCat::Ui => "[  UI]: ",
        }
    }
}

pub fn log_internal(cat: LogCat, level: LogLevel, args: std::fmt::Arguments) {
    if level >= LogCat::default_level(&cat) {
        let log_fun = match level {
            LogLevel::Debug => web_sys::console::debug_1,
            LogLevel::Info => web_sys::console::info_1,
            LogLevel::Warning => web_sys::console::warn_1,
            LogLevel::Error => web_sys::console::error_1,
        };

        log_fun(&format!("{}{}", cat.to_string(), args.to_string()).into());
    }
}

macro_rules! debug {
    ($cat:expr, $($args:tt)+) => ({
        $crate::utils::log::log_internal(
            $cat,
            $crate::utils::log::LogLevel::Debug,
            format_args!($($args)+)
        );
    });
}

macro_rules! info {
    ($cat:expr, $($args:tt)+) => ({
        $crate::utils::log::log_internal(
            $cat,
            $crate::utils::log::LogLevel::Info,
            format_args!($($args)+)
        );
    });
}

macro_rules! warning {
    ($cat:expr, $($args:tt)+) => ({
        $crate::utils::log::log_internal(
            $cat,
            $crate::utils::log::LogLevel::Warning,
            format_args!($($args)+)
        );
    });
}

macro_rules! error {
    ($cat:expr, $($args:tt)+) => ({
        $crate::utils::log::log_internal(
            $cat,
            $crate::utils::log::LogLevel::Error,
            format_args!($($args)+)
        );
    });
}

pub(crate) use debug;
pub(crate) use error;
pub(crate) use info;
pub(crate) use warning;

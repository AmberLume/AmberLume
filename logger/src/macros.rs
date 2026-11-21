#[macro_export]
macro_rules! log {
    ($level:expr, $($argument:tt)+) => {
        $crate::Logger::log(&$crate::LogRecord {
            level: $level,
            target: ::std::module_path!(),
            message: ::std::format_args!($($argument)+),
            file: ::std::file!(),
            line: ::std::line!(),
        })
    };
}

#[macro_export]
macro_rules! trace {
    ($($argument:tt)+) => {
        $crate::log!($crate::LogLevel::Trace, $($argument)+)
    };
}

#[macro_export]
macro_rules! debug {
    ($($argument:tt)+) => {
        $crate::log!($crate::LogLevel::Debug, $($argument)+)
    };
}

#[macro_export]
macro_rules! info {
    ($($argument:tt)+) => {
        $crate::log!($crate::LogLevel::Info, $($argument)+)
    };
}

#[macro_export]
macro_rules! warn {
    ($($argument:tt)+) => {
        $crate::log!($crate::LogLevel::Warn, $($argument)+)
    };
}

#[macro_export]
macro_rules! error {
    ($($argument:tt)+) => {
        $crate::log!($crate::LogLevel::Error, $($argument)+)
    };
}

#[macro_export]
macro_rules! fatal {
    ($($argument:tt)+) => {{
        let message = ::std::format!($($argument)+);
        $crate::log!($crate::LogLevel::Fatal, "{}", message);
        ::std::panic!("{}", message)
    }};
}

#[macro_export]
macro_rules! check {
    ($condition:expr $(,)?) => {
        $crate::check!($condition, "check failed: {}", ::std::stringify!($condition))
    };
    ($condition:expr, $($argument:tt)+) => {
        if $crate::VALIDATION && !$condition {
            $crate::fatal!($($argument)+)
        }
    };
}

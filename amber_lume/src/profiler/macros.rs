#[macro_export]
macro_rules! profile_cpu_zone {
    ($profiler:expr, $name:literal, $body:block) => {{
        let _guard = $profiler.cpu_zone($name);
        $body
    }};
}

#[macro_export]
macro_rules! profile_cpu_meta {
    ($profiler:expr, $name:literal, $value:expr) => {{
        $profiler.cpu_meta($name, ::core::convert::Into::into($value))
    }};
}

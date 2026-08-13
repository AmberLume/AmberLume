#[macro_export]
macro_rules! profile_cpu_zone {
    ($profiler:expr, $name:expr, $body:block) => {{
        let _guard = $profiler.cpu_zone($name);
        $body
    }};
}

#[macro_export]
macro_rules! profile_cpu_meta {
    ($profiler:expr, $name:expr, $value:expr) => {{
        $profiler.cpu_meta($name, ::core::convert::Into::into($value))
    }};
}

#[macro_export]
macro_rules! profile_gpu_zone {
    ($profiler:expr, $cmd:expr, $name:expr, $body:block) => {{
        let _guard = $profiler.gpu_zone($cmd, $name);
        $body
    }};
}

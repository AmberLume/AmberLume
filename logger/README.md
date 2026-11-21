[← AmberLume](../README.md)

# logger

Logging for the whole project: macros with a level and a category, one level filter, outputs supplied by the platform, a fatal path and debug checks.

---

## Why

- **One front for every crate.** Engine and app crates log through these macros and do not know where the records go.
- **No platform code.** The crate holds the records, the filter and the fan-out to outputs. Writing to a console, logcat or a file is up to the platform: it implements `LogOutput` and hands the outputs over at startup. The crate depends on neither `tracing` nor `log`.
- **Failures go through the log.** `fatal!` and `check!` write the message to every output before the process stops.
- **Process-wide.** The logger is one static, set once at startup (as in `log`, `tracing` or UE's `GLog`), so any crate can log without passing a handle around.

---

## How

### Initialize

The platform calls this once, first thing in `main`:

```rust
Logger::initialize(LogLevel::Info, vec![Box::new(ConsoleLogOutput)])?;
```

- A second call returns an error.
- Records logged before it are dropped.
- The level is fixed here; `RUST_LOG` is not read.

### Log

```rust
use logger::{info, warn};

info!("Swapchain created: {}x{}", width, height);
warn!("unknown token: {}", token);
```

- Levels, lowest first: `trace!`, `debug!`, `info!`, `warn!`, `error!`. `log!(level, ...)` takes the level as a value.
- Arguments use `format!` syntax. They are evaluated only when the level passes the filter.
- The category (`target`) is the calling module (`module_path!()`). File and line are recorded with it.

### Fatal

`fatal!("...")` logs at `Fatal`, then panics with the same message. It works in every build.

### Checks

`check!(condition)` or `check!(condition, "...", arguments)` calls `fatal!` when the condition is false.

- On in debug builds.
- In release, turn on the `validation` feature: `cargo build --release -p desktop --features logger/validation`.
- When off, the condition is not evaluated.

### Outputs

```rust
pub trait LogOutput: Send + Sync {
    fn write(&self, record: &LogRecord);
}
```

`LogRecord` carries `level`, `target`, `message` (`fmt::Arguments`), `file` and `line`. `file` and `line` are `Option`, because records forwarded from other logging crates may not have them.

### Forwarding other logging crates

Third-party crates log through `log` or `tracing`. A platform forwards their records by calling `Logger::log` with a `LogRecord`. `Logger::enabled(level)` and `Logger::level()` let it filter before building one: desktop's `LogBridge` sets the `log` max level from `Logger::level()`, so filtered `log` calls stop at the call site.

---

## Where logs go

| Crate | Output | Forwarded |
|---|---|---|
| `lume/desktop` | `ConsoleLogOutput`: stdout, one line per record | `log` through `LogBridge` (calloop, gpu-allocator, rapier, …), `tracing` through `TracingBridge` (winit) |
| `lume/android` | Not on the logger yet: its own `tracing` setup writes to logcat, engine records are lost until ALume-256 | — |
| `builder` | Not on the logger: its own `tracing` setup (`builder/src/tracing.rs`) | — |

Desktop line format, UTC time, no colors:

```
23:16:31.976104  INFO main gpu::device::vulkan_context: VulkanContext created
```

Vulkan validation messages bypass the logger: there is no debug messenger yet, so the layer prints straight to stdout/stderr.

# smoke-run

Build the `desktop` app and run it for a few seconds to confirm it **compiles,
starts, and renders** — then auto-close it.

## Why

The app has no built-in "exit after N seconds/frames" and renders continuously
(`about_to_wait` always requests a redraw), so a quick "does it still work" check
needs an external time box. This script builds, launches from the correct working
directory, bounds the run with `timeout`, captures the log, and interprets the
exit code so you get a clear OK/FAIL.

## Usage

```bash
ai_tools/smoke-run.sh                 # build + run ~5s, auto-detect backend
ai_tools/smoke-run.sh --secs 8        # longer window
ai_tools/smoke-run.sh --x11           # force X11 winit backend
ai_tools/smoke-run.sh --wayland       # force Wayland winit backend
ai_tools/smoke-run.sh --log out.log   # custom log path
```

Default log: `target/ai_tools-smoke-run.log`.

## What it does

1. Pick a winit backend: `--x11` / `--wayland`, else auto-detect from
   `$WAYLAND_DISPLAY` / `$DISPLAY` (Wayland is the default build).
2. `cargo build -p desktop` (compile time is kept **outside** the timed window).
3. Run `target/build/debug/desktop` from `target/distribution` (required: assets
   resolve relative to the current directory) under
   `timeout --signal=TERM <secs>`, redirecting stdout+stderr to the log.
4. Print the last log lines, check for the `AmberLume created` signal, and report.

## Interpreting the result

- **`RESULT: OK`** — `timeout` exit `124` (it had to kill the app = the app
  survived the whole window and kept rendering) or `0` (clean self-exit).
- **`RESULT: FAIL`** — the app exited early; the script greps the log for
  `panic` / `error` / `No suitable device` / `VK_ERROR`.

Healthy startup log chain (INFO):
`VulkanContext created` → `DeviceContext created` → `Swapchain created` →
`SurfaceRenderTarget created` → `AmberLume created` → render target attached.
There is **no** explicit "first frame presented" line; success is inferred from
this chain plus the absence of a draw panic.

## Gotchas

- **Requires a live graphical session.** With no display (headless CI / sandbox),
  surface creation fails and the script reports FAIL — that is environmental, not
  a code regression. Run on a machine with a desktop session.
- **External `timeout`/SIGTERM skips graceful shutdown**, so you may see a few
  teardown warnings at the end — expected. For a clean close on X11 you can
  instead `wmctrl -c AmberLume`.
- **Binary path is `target/build/debug/desktop`** (target-dir override), not
  `target/debug`.
- If `target/distribution/assets` is missing, run
  [`build-assets.sh`](build-assets.md) first.
- `RUST_LOG` does not raise verbosity yet (`lume/core` uses `fmt().init()` with a
  fixed `INFO` level).

## Related files

- `lume/desktop/src/main.rs` — entry point, window, validation env parsing.
- `lume/desktop/src/application.rs` — winit event loop, redraw, close handling.
- `lume/desktop/src/platform_providers/desktop_io_provider.rs` — cwd-relative
  asset resolution.
- `.cargo/config.toml` — `target-dir = target/build`.

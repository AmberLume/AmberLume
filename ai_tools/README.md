# `ai_tools/` — Agent Tooling for AmberLume

Helper scripts and notes for an **AI agent** (or any developer) working on
AmberLume. The goal is to stop re-discovering *how to build / run / validate*
the project every session: the commands and their gotchas live here.

All scripts resolve the workspace root from their own location, so they can be
invoked from anywhere:

```bash
ai_tools/build-assets.sh
ai_tools/smoke-run.sh
ai_tools/vk-run.sh
```

---

## Tool registry

| Tool | Script | What it does | Guide |
|------|--------|--------------|-------|
| **build-assets** | `build-assets.sh` | Run the asset/shader builder (`cargo run -p builder`); regenerate `target/generated/resources.rs`; optionally grep it for a generated symbol. | [docs/build-assets.md](docs/build-assets.md) |
| **smoke-run** | `smoke-run.sh` | Build the `desktop` app and run it for a few seconds (auto-closed via `timeout`); capture logs; report whether it started and rendered. | [docs/smoke-run.md](docs/smoke-run.md) |
| **vk-run** | `vk-run.sh` | Same launch with **all Vulkan validation** enabled, then scan the log for validation findings. | [docs/vk-run.md](docs/vk-run.md) |

---

## Project quick facts (the gotchas these tools encode)

- **Builder is manual.** `cargo run -p builder` is **not** wired into `cargo
  build` (the only `build.rs` just touches an empty manifest). Re-run it after
  adding/renaming any shader or asset, or the engine references **stale**
  constants. See [docs/build-assets.md](docs/build-assets.md).
- **Generated symbols live in** `target/generated/resources.rs` — read it to find
  a constant name (e.g. `shaders::MAIN_FRAG`). Names are the file stem,
  uppercased, non-alphanumerics → `_`, duplicates suffixed `_0/_1`.
- **Build output dir is `target/build`** (overridden in `.cargo/config.toml`), so
  the desktop binary is `target/build/debug/desktop`, **not** `target/debug`.
- **Run from `target/distribution`.** The desktop I/O provider resolves assets
  relative to the current directory (`current_dir()/assets`); running from
  anywhere else fails to load assets.
- **No timed/headless exit exists.** The window only closes on user request, so
  smoke runs are bounded externally with `timeout` (`exit 124` = it survived).
- **Vulkan validation bypasses logging.** There is no debug messenger; the
  validation layer prints to stdout/stderr. Capture the process output and grep
  the markers — errors: `Validation Error` / `VUID-` / `SYNC-`; warnings:
  `Validation Warning` / `Validation Performance Warning` / `UNASSIGNED-BestPractices-`.
  Note: setup advisories also come as `Validation Warning` (`VALIDATION-SETTINGS`,
  `WARNING-Setting-Limit-Adjusted`) and are benign.
- **`RUST_LOG` is currently ignored** in both `lume/core` (`fmt().init()` with no
  env filter, fixed `INFO`) and the builder (hard-coded `builder=trace`).

---

## Planned (not yet implemented)

- **`AMBERLUME_PERF` log flag** — dump the existing per-frame `FrameProfile`
  (the same CPU/GPU timings shown in the debug overlay) to the log on a flag,
  emitted right next to where the on-screen statistics are collected. No
  cross-frame accumulation — single-frame snapshot only. Requires a small engine
  edit; a `perf-run.sh` wrapper + guide will be added here once it lands.

---

## Maintaining this directory

When you add a new agent tool:

1. Drop the script in `ai_tools/` (resolve the workspace root from
   `${BASH_SOURCE[0]}`; keep a usage header comment).
2. Add a guide at `ai_tools/docs/<tool>.md` (purpose, when to use, usage,
   output interpretation, gotchas).
3. Add a row to the **Tool registry** table above.
4. If it changes how agents build/run/validate, mention it in the root
   `AGENTS.md` (§8).

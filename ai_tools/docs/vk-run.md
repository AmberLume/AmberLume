# vk-run

Run the `desktop` app with **all Vulkan validation** enabled and scan the log for
validation findings.

## Why

`AMBERLUME_VK_VALIDATION` enables the Khronos validation layer plus feature
checks, but the engine installs **no debug messenger** — the layer prints its
messages straight to stdout/stderr, bypassing the app's logging. So to catch
validation problems you must capture the process output and grep for the layer's
markers. This script does both: it runs the smoke launch with validation on, then
buckets the findings.

## Usage

```bash
ai_tools/vk-run.sh                              # all 3 tokens, ~8s
ai_tools/vk-run.sh --secs 12                    # longer window
ai_tools/vk-run.sh --tokens synchronization     # a single feature only
ai_tools/vk-run.sh --x11                         # force backend
```

Default tokens: `synchronization,best_practices,gpu_assisted`.
Default log: `target/ai_tools-vk-run.log`.

## Validation tokens

Parsed in `lume/desktop/src/main.rs` (`parse_validation_env`), comma-separated:

| Token | Vulkan feature | Catches |
|-------|----------------|---------|
| `synchronization` | `SYNCHRONIZATION_VALIDATION` | Missing/incorrect barriers, hazards (`SYNC-*`). |
| `best_practices` | `BEST_PRACTICES` | API anti-patterns (`UNASSIGNED-BestPractices-*`, advisory). |
| `gpu_assisted` | `GPU_ASSISTED` | Runtime, shader-side access checks (GPU-AV). |

Note: setting the env var (even with an unknown/typo token) still enables the base
validation layer; only recognized tokens enable their feature checks. Unknown
tokens emit a `warn!` and are ignored.

## What it does

1. Export `AMBERLUME_VK_VALIDATION=<tokens>` and call
   [`smoke-run.sh`](smoke-run.md) (same build + timed run + log capture).
2. Grep the captured log into three buckets:
   - **Errors / hazards**: `Validation Error`, `VUID-`, `SYNC-` — must review.
   - **Warnings**: `Validation Warning`, `Validation Performance Warning`,
     `UNASSIGNED-BestPractices-` — review (best-practices + real warnings).
   - **Benign setup advisories** (suppressed, only counted): `VALIDATION-SETTINGS`,
     `WARNING-Setting-Limit-Adjusted`, and the "GPU-AV + Core Check is slow" note.
     These always appear when all tokens are on and are not defects.

## Interpreting the result

- **Exit code reflects the run** (0 = app survived, 1 = it crashed) — *not* the
  presence of validation messages, which are surfaced as text for you to judge.
- **Any `Validation Error` / `VUID-` / `SYNC-` is a real issue** to fix; the final
  `RESULT:` line shouts if errors are present.
- Warnings are worth reading; best-practices ones may appear even in a healthy app.
- The validation layer prints **`Validation Warning`** for setup messages too (e.g.
  GPU-AV auto-disabling ray-query / mesh-shading checks because the GPU lacks those
  features) — these are auto-suppressed into the benign bucket.
- GPU-assisted validation slows startup noticeably — hence the larger default
  window (8s). Increase with `--secs` if init doesn't complete in time.

## Gotchas

- Same environment requirement as smoke-run: a **live graphical session**.
- Teardown via SIGTERM can add a few validation warnings at shutdown — those are
  artifacts of the forced exit, not necessarily real defects.

## Related files

- `lume/desktop/src/main.rs` — `parse_validation_env`.
- `amber_lume/src/render/device/validation_features.rs` — token → feature mapping.
- `amber_lume/src/render/device/vulkan_context.rs` — instance/layer wiring.
- See also: there is **no** `vkCreateDebugUtilsMessengerEXT` in the codebase yet;
  adding one (forwarding to `tracing`) would make this scan deterministic instead
  of grep-based.

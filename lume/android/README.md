[← AmberLume](../../README.md) · [lume/core](../core/README.md) · [lume/desktop](../desktop/README.md)

# lume/android

Android platform target for AmberLume.

---

## What it does

Provides `SurfaceProvider` and `IOProvider` implementations for Android, and bridges the Android activity lifecycle to the engine. Built as a `cdylib` (`lume_android`) loaded by a Gradle/Java activity via `android-activity` with the `game-activity` backend.

---

## How it works

`lib.rs` registers the native entry point expected by `android-activity`. On `Resume`, it constructs platform providers and calls `Lume::create` from `lume/core`. The main loop runs until the activity is destroyed, routing touch and lifecycle events to the engine.

All application logic lives in `lume/core`; this crate is a thin adapter between the Android NDK and the engine traits.

---

## Requirements

### Vulkan 1.3

The Android target requires **Vulkan 1.3**. The renderer relies on features promoted to core in 1.3 (dynamic rendering, synchronization2, and related extensions). Devices without Vulkan 1.3 support are not compatible.

In practice this means:

- **Real hardware is required.** Android emulators do not expose Vulkan 1.3.
- Devices from roughly 2022 and newer with a flagship SoC are the target.

Support for Vulkan versions below 1.3 is not planned in the current architecture. Adding it would require significant changes to the render backend.

---

## Building

The Android build uses Gradle. The Rust library is compiled via the standard Android NDK toolchain and bundled by Gradle.

```bash
cd lume/android
./gradlew assembleDebug
```

Deploy to a connected device:

```bash
./gradlew installDebug
```

A physical device with Vulkan 1.3 support must be connected.

---

## Performance

For detailed per-pass breakdowns, startup times, asset loading times, and GPU frame stats see [`lume/desktop/README.md`](../desktop/README.md) — the render pipeline is identical. Android timings are not broken down at the same granularity: mobile GPUs throttle aggressively and timings vary several times between frames, making fine-grained measurements unreliable.

Tested on **Samsung Galaxy Tab S11 Ultra** (fullscreen). All values in µs; CPU prep/commands are recorded once per pass, dispatch is GPU time.

| Pass | Prep | Commands | Dispatch | Notes |
|---|---|---|---|---|
| Culling | ~6 | ~10 | ~68 | per-view frustum culling for the main camera |
| Cascade culling indirect | ~0 | ~4 | ~92 | one indirect cull dispatch covering every cascade view |
| Skinning | ~1 | ~5 | ~411 | compute pass; writes final bone transforms |
| Cascade compute | ~0 | ~3 | ~25 | derives cascade splits from the SDSM depth reduction |
| SDSM | ~0 | ~5 | ~118 | parallel reduction over the main depth buffer; stride 4 keeps cost down |
| Shadows | ~0 | ~24 | ~615 | single multiview draw into a layered shadow map |
| Main | ~0 | ~16 | ~2190 | depth prepass + lit forward pass; shadow filtering happens inline |
| Physics debug | ~7 | ~5 | ~121 | unchanged in this pass; collect spikes due to mapping and uploading line geometry per entity |
| UI | ~8 | ~17 | ~165 | dispatch scales with UI complexity and window size |
| **Total** | **~22** | **~89** | **~4056** | **~4.2 ms full frame** |

> These numbers are captured at the default performance hint. Requesting maximum performance from the OS drops the GPU frame to **~1.2 ms** at the cost of higher power draw and thermals — useful for benchmarking, not for sustained sessions on battery.

Shadow quality is the main lever: reducing cascade count or resolution saves a few hundred µs with minimal visual impact.

**Memory: ~1.4 GB RAM.** Headroom can be recovered by:
- reducing frames-in-flight
- shrinking per-frame staging/vertex/index buffers
- dropping shadow cascades

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

Tested on **Samsung Galaxy Tab S11 Ultra** (fullscreen):

| Stage | Time | Notes |
|---|---|---|
| ECS — data preparation (indices, etc.) | ~100 µs | |
| Render command collection | 250–300 µs | |
| Culling | ~80 µs | |
| Skinning | ~170 µs | |
| Depth prepass | ~220 µs | switching to D16 format should reduce this |
| Shadow map | ~1 ms | scene- and settings-dependent |
| Main pass | ~1.2 ms | shadow quality is the main lever |
| Physics debug | 80 µs – 1 ms | scales with entity count; current overhead is negligible in practice |
| UI | ~90 µs | |
| **Full frame** | **~5 ms** | |

Shadow quality is the main lever: reducing cascade count or resolution saves ~400 µs with minimal visual impact.

**Memory: ~1.4 GB RAM.** Headroom can be recovered by:
- reducing frames-in-flight
- shrinking per-frame staging/vertex/index buffers
- dropping shadow cascades

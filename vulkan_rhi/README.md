[← AmberLume](../README.md)

# vulkan_rhi

The Vulkan implementation of [`explicit_rhi`](../explicit_rhi/README.md), written from scratch on `ash`. It does not depend on `gpu`.

---

## Why

- **Vulkan in one place.** Everything that calls Vulkan for the explicit renderer lives here.
- **Only the contract is public.** The crate exports `Vulkan`, the implementation of `ExplicitRhi`. Its types are reached through `Vulkan`'s associated types and their methods come from `explicit_rhi` traits; their own methods are `pub(crate)`, so nothing above can call Vulkan past the contract.

---

## How

```rust
let renderer = ExplicitRender::<vulkan_rhi::Vulkan>::create(settings)?;
```

Inside the renderer the device is `Backend::Device`.

### Instance

- Vulkan 1.3.
- Surface extensions: whichever of `surface`, `wayland_surface`, `xlib_surface`, `xcb_surface`, `win32_surface`, `android_surface` and `metal_surface` the loader offers, so a surface can be created for any window later. Plus `swapchain_colorspace` and `debug_utils` when available.
- Validation: any `Validation` flag enables `VK_LAYER_KHRONOS_validation`. `synchronization`, `best_practices` and `gpu_assisted` become its validation features. A missing layer or feature is a warning, not an error.
- With validation on, a debug messenger sends errors and warnings to `logger`.

### Physical device

A device is suitable when it supports Vulkan 1.3, `VK_KHR_swapchain` and every required feature, and has a graphics queue family. A discrete GPU wins over the others. The log names every device and why it was skipped.

Required features live in one place, `DeviceFeatures::REQUIRED`: the same list is checked when a device is picked and enabled when it is created. `DeviceFeatures` turns Vulkan's `Bool32` into `bool`.

### Device

- One queue from the graphics family.
- Enabled: `VK_KHR_swapchain`, `dynamicRendering`, `synchronization2`. A new feature joins `DeviceFeatures` together with the first code that needs it.
- `destroy` waits for the device to be idle, then destroys the device and the instance.

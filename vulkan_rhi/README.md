[← AmberLume](../README.md)

# vulkan_rhi

The Vulkan implementation of [`explicit_rhi`](../explicit_rhi/README.md), written from scratch on `ash`. It does not depend on `gpu`.

---

## Why

- **Vulkan in one place.** Everything that calls Vulkan for the explicit renderer lives here.
- **Only the contract is public.** The crate exports `VulkanRhi`, which implements `ExplicitRhi`. Its parts are `pub(crate)`, so nothing above can call Vulkan past the contract.

---

## How

```rust
let renderer = ExplicitRender::<vulkan_rhi::VulkanRhi>::create(settings)?;
```

`VulkanRhi` holds the `VulkanInstance`, the `PhysicalDevice`, the `VulkanDevice` side by side, the current `PresentSettings` (`default()` until set) and, while a window is attached, a `VulkanSurface`. Each part takes only the parts it uses as arguments. Every part owns one piece of logic, as in `gpu`:

| Part | Owns |
|---|---|
| `instance/` | `VulkanInstance` (enabled extensions, `check_surface`), validation layer, debug messenger, the surface loader |
| `device/` | `PhysicalDevice` (selection, `check_surface`), `VulkanDevice` (the logical device, the swapchain loader, `wait_idle`) |
| `surface/` | `VulkanSurface`: the attached window as one object — the surface from the window handles (checked for present support), its size, the config it wants and the swapchain it has. It owns `resize`, `set_present_settings`, the rebuild and the destruction order, and keeps a copy of the surface loader and the physical device handle to query itself |
| `swapchain/` | `SwapchainConfig` picked from `PresentSettings` by one file per value (`surface_format`, `surface_present_mode`, `surface_composite_alpha`, `surface_extent`, `surface_image_count`, `surface_image_usage`) in `select`, `VulkanSwapchain` (the swapchain, its config and images) built from a config and the old swapchain, `SwapchainImage` with its view |

### Instance

- Vulkan 1.3.
- Surface extensions: whichever of `surface`, `wayland_surface`, `xlib_surface`, `xcb_surface`, `win32_surface`, `android_surface` and `metal_surface` the loader offers, so a surface can be created for any window later. Plus `swapchain_colorspace` and `debug_utils` when available. The enabled ones are kept: `VulkanInstance::check_surface` fails `attach` when the extensions a window needs (from `ash-window`) are not enabled, for example on a machine without a window system.
- Validation: any `Validation` flag enables `VK_LAYER_KHRONOS_validation`. `synchronization`, `best_practices` and `gpu_assisted` become its validation features. A missing layer or feature is a warning, not an error.
- With validation on, a debug messenger sends errors and warnings to `logger`. A failed messenger destroys the instance and fails creation.

### Physical device

A device is suitable when it supports Vulkan 1.3 and `VK_KHR_swapchain`, and has a graphics queue family. A discrete GPU wins over the others. The log names every device and why it was skipped.

### Device

- One queue from the graphics family.
- Enabled: `VK_KHR_swapchain`, and `dynamicRendering` and `synchronization2` requested in `PhysicalDeviceVulkan13Features`. Vulkan checks them itself: device creation fails when one is not supported. A new feature is requested there together with the first code that needs it.
- `destroy` waits for the device to be idle, then destroys it; a failed wait still destroys it and is returned after. `VulkanRhi::destroy` destroys the instance after the device.

### Window (`attach`, `detach`, `resize`, `set_present_settings`)

- `attach` creates a `VulkanSurface` with the current present settings: the surface, a `SwapchainConfig` from the window size and the settings, and a `VulkanSwapchain` from it. A window of zero size cannot be attached. Any failure after the surface is created destroys it.
- The surface is created from the window's raw handles with `ash-window`, after `VulkanInstance::check_surface`. The graphics queue family must be able to present to it (`PhysicalDevice::check_surface`); otherwise `attach` fails. Images are `EXCLUSIVE` to that family.
- `SwapchainConfig::select` takes every value from `PresentSettings`: the first supported format with its color space, present mode and composite alpha, the requested usage if the surface supports it, the image count clamped to the surface limits. The size is the surface's current extent; where the surface leaves it to the application (Wayland), the window size clamped to the surface limits; a zero window size gives a zero extent. Nothing has a default or a fallback: an unsupported request is an error and changes nothing.
- A swapchain lives shorter than its surface: a resize or settings change that gives a different non-zero config builds a new one from the old one. `VulkanSurface` keeps the config it wants and the swapchain it has; they differ only while the window has a zero size.
- Without a window `resize` does nothing and `set_present_settings` only stores the settings. With one:
  - `resize` stores the size and takes the wanted config with a new extent and transform; the present settings are not needed, the values picked from them are already in the config;
  - `set_present_settings` picks a new config with `SwapchainConfig::select`; an error there changes nothing, and `VulkanRhi` stores the settings once the rebuild succeeds;
  - both then rebuild:
    - a lost swapchain (see below) is an error at any size;
    - a zero extent keeps the current swapchain; the new config is kept and applies on the next non-zero size;
    - a config equal to the current swapchain's is only kept;
    - otherwise the device is waited idle, `VulkanSwapchain::create(old, ...)` builds the new swapchain from the old one (`old_swapchain`), and the old one is destroyed;
    - the new config is kept only when the rebuild got this far: a failed wait leaves the config and the swapchain as they were (`resize` has already stored the size).
- A failed `VulkanSwapchain::create` (`vkCreateSwapchainKHR`, `vkGetSwapchainImagesKHR` or an image view) retires the old swapchain even so: it is destroyed and the surface keeps no swapchain, so `resize` and `set_present_settings` are errors until the window is detached and attached again.
- Destruction order: wait for the device to be idle, destroy the image views, the swapchain, then the surface. `VulkanRhi::destroy` destroys the surface, then the device, then the instance.
- The extension loaders live as long as what they are loaded from: the surface loader with the instance (each `VulkanSurface` keeps a copy), the swapchain loader with the device.
- Fixed by the Vulkan model, not a choice yet: one array layer, `EXCLUSIVE` sharing on the graphics family, `clipped`, `pre_transform` = the surface's current transform.

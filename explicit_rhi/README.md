[← AmberLume](../README.md)

# explicit_rhi

The explicit RHI: one trait that every low-level graphics API backend implements for the explicit renderer. Targets are Vulkan, DX12 and Metal; `vulkan_rhi` implements it now, others can be added as their own crates. No implementation and no API types here.

---

## Why

- **The explicit renderer does not know Vulkan.** It asks the explicit RHI to show frames in a window, to create a buffer, a pipeline. A new capability is added here first, then implemented in each backend, then used by `explicit_render`.
- **One object per backend.** A backend is one type that holds its own state. How it is built inside (Vulkan: instance, device, swapchain; DX12: factory, adapter, device, `IDXGISwapChain`; Metal: device, `CAMetalLayer`) is its own business: backends are not forced into common object traits, because their objects never meet.
- **The backend is a type parameter of the renderer.** `explicit_render` is `ExplicitRender<Rhi: ExplicitRhi>`. Every call is static: no `dyn`, no casts.
- **Build flags decide what is compiled, startup flags pick from it.** A build can carry several backends; each compiled backend is its own copy of the explicit renderer.
- **Explicit APIs only:** memory, synchronization, buffer addresses and bindless are ours to manage. Where the APIs differ in model, the contract states intent: synchronization is a usage word, rendering into targets is "begin rendering with these targets and clears". Implicit APIs go to the implicit renderer.
- **The crate knows only itself.** It has its own window and present mode types; the explicit renderer converts the `render` types into them.

---

## How

```rust
pub struct ExplicitRender<Rhi: ExplicitRhi> {
    rhi: Rhi,
}

let mut rhi = Rhi::create(ExplicitRhiSettings { validation: Validation::OFF })?;

rhi.set_present_settings(present_settings)?;
rhi.attach(window)?;
rhi.resize(size)?;
rhi.detach()?;

rhi.destroy()?;
```

The engine picks a compiled backend: `ExplicitRender::<vulkan_rhi::VulkanRhi>`.

### `ExplicitRhi`

| Method | Purpose |
|---|---|
| `create(settings)` | Connect to the GPU |
| `attach(window)` | Where to show frames: a window (its surface) with its size. Uses the current present settings. One window at a time; a second window or a window of zero size is an error |
| `detach()` | Stop showing frames in it. Without an attached window it is an error |
| `resize(size)` | The window's size changed. The size belongs to the attached surface: without one there is no size and nothing to resize, rendering goes on headless and frames are taken by readback. A zero size (minimized) keeps the current images until a non-zero size |
| `set_present_settings(settings)` | Set how frames are shown. Until it is called, `PresentSettings::default()` applies. Without a window they are kept for the next `attach`; with one the window is rebuilt; while it is minimized they wait for a non-zero size |
| `destroy()` | Release everything, the window's swapchain first |

- Everything about showing frames is `PresentSettings`: defaults in `PresentSettings::default()`, concrete settings from the renderer. The backend has no defaults and no fallbacks of its own. Lists are in order of preference: the backend takes the first entry the window supports, none supported is an error and leaves the window as it was.
- A size or settings change that ends up with the same result rebuilds nothing.
- Present settings are passed once, through `set_present_settings`; `attach` and later attaches use the last ones set. `resize` does not take them: what was picked from them stays, only what comes from the window changes (its size and, in Vulkan, the surface transform). Settings the attached window cannot use are an error and are not kept.
- `resize` without a window does nothing and is not an error. After a failed swapchain rebuild (in `resize` or `set_present_settings`) `resize` and `set_present_settings` are errors, at any size, until the window is detached and attached again. A failed `attach` attaches nothing.
- How a window is shown is the backend's business: Vulkan and DX12 build a swapchain, Metal a layer.
- Objects created later (buffers, images, pipelines) are typed handles: an index and a generation. The data lives in the backend's own tables, as the bindless tables do now.

### Types

```rust
ExplicitRhiSettings { validation: Validation }
Validation { api, synchronization, best_practices, gpu_assisted }
Window { display, window, size: WindowSize }
WindowSize { width, height }
PresentSettings { formats: Vec<PresentFormat>, present_modes: Vec<PresentMode>, composite_alphas: Vec<CompositeAlpha>, image_count, usage: ImageUsage }
PresentFormat { format: Format, color_space: ColorSpace }
PresentMode { Fifo, Mailbox, Immediate }
CompositeAlpha { Opaque, PreMultiplied, PostMultiplied, Inherit }
Format { Bgra8UnormSrgb, Rgba8UnormSrgb }
ColorSpace { SrgbNonlinear }
ImageUsage { color_attachment }
```

- `Validation::OFF` turns everything off. Any flag turns API validation on. Each backend maps the flags to its own tools and ignores a flag its API has no tool for.
- `Format` names formats the way DXGI, Metal and WebGPU do. `Format`, `ColorSpace` and `ImageUsage` hold only what is in use; images and render targets will use the same types.
- `PresentSettings::default()`: `Bgra8UnormSrgb`, then `Rgba8UnormSrgb`, both `SrgbNonlinear`; `Fifo`; `Opaque`, `Inherit`, `PreMultiplied`, `PostMultiplied`; 3 images; color attachment.
- `image_count` is clamped to what the window allows.
- `Window` carries the raw handles from `raw-window-handle`. The platform keeps the window alive while it is attached.

### What each backend builds

| | Vulkan | DX12 | Metal |
|---|---|---|---|
| `create` | instance, physical device, device, graphics queue | DXGI factory, adapter, `ID3D12Device`, command queue | `MTLDevice`, command queue |
| `Validation` | Khronos validation layer and its features | debug layer, GPU-based validation | API validation, shader validation |
| `attach`, `resize` | surface + `VkSwapchainKHR`, recreated from the old one | `IDXGISwapChain` from the DXGI factory and the command queue, `ResizeBuffers` | `CAMetalLayer`, a drawable taken every frame |

MoltenVK runs the Vulkan backend on Metal; a native Metal backend would be its own crate.

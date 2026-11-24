[← AmberLume](../README.md)

# explicit_rhi

The explicit RHI: the common contract of low-level graphics APIs for the explicit renderer. Targets are Vulkan, DX12 and Metal; `vulkan_rhi` implements it now, others can be added as their own crates. No implementation and no API types here.

---

## Why

- **The explicit renderer does not know Vulkan.** It works with the explicit RHI: asks it for a device, a buffer, a swapchain. A new capability is added here first, then implemented in each backend, then used by `explicit_render`.
- **Backends do not know each other.** Each lives in its own crate and implements the same traits with its own types.
- **The backend is a type parameter of the renderer.** `explicit_render` is written as `ExplicitRender<Backend: ExplicitRhi>`. Every call inside is static: no `dyn`, no casts.
- **Build flags decide what is compiled, startup flags pick from it.** A build can carry several backends; each compiled backend is its own copy of the explicit renderer. The engine picks one at startup.
- **Objects are what the renderer asks for,** not one API's object list. Each backend builds an object from as many native objects as its API needs.
- **Where the APIs differ in model, the contract states intent.** Synchronization is a usage word, not a barrier; rendering into targets is "begin rendering with these targets and clears", not a render pass object.
- **Explicit APIs only:** memory, synchronization, buffer addresses and bindless are ours to manage. Implicit APIs go to the implicit renderer.

---

## How

```rust
pub struct ExplicitRender<Backend: ExplicitRhi> {
    device: Backend::Device,
}

impl<Backend: ExplicitRhi> ExplicitRender<Backend> {
    pub fn create(settings: DeviceSettings) -> Result<Self> {
        let device = Backend::Device::create(settings)?;

        Ok(Self { device })
    }
}
```

The engine picks a compiled backend:

```rust
let renderer: Box<dyn Render> = Box::new(ExplicitRender::<vulkan_rhi::Vulkan>::create(settings)?);
```

### Traits

| Trait | Purpose |
|---|---|
| `ExplicitRhi` | One backend: the types it implements. Now `type Device` |
| `Device` | `create(settings)` and `destroy()`. Owns everything the backend needs to talk to the GPU |

### Mapping to target APIs

| Object | Vulkan | DX12 | Metal |
|---|---|---|---|
| `Device` | instance, physical device, device, graphics queue | DXGI factory, adapter, `ID3D12Device`, command queue | `MTLDevice`, command queue |
| `Validation` | Khronos validation layer and its features | debug layer, GPU-based validation | API validation, shader validation |

MoltenVK runs the Vulkan backend on Metal; a native Metal backend would be its own crate.

### Settings

```rust
DeviceSettings { validation: Validation }
Validation { api, synchronization, best_practices, gpu_assisted }
```

- `Validation::OFF` turns everything off. Any flag turns API validation on.
- Each backend maps the flags to its own tools and ignores a flag its API has no tool for (DX12 has no best practices layer).

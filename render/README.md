[← AmberLume](../README.md)

# render

The interface between the engine and a renderer. Only the interface: no implementation and no GPU types.

---

## Why

- **The engine holds a renderer, not a GPU.** `AmberLume` tells the renderer what to prepare for and hands it frames. How the frame is drawn is the renderer's business.
- **Renderers are split by the API model.** The explicit renderer works on explicit APIs (Vulkan), an implicit one on implicit APIs (WebGPU, WebGL2). Both implement this trait and know nothing about each other.
- **The engine picks one.** A renderer is created through its own type, chosen by the engine at startup by a flag or by the build. It is not switched on the fly.
- **Nothing GPU-specific crosses.** The crate depends on `render_snapshot` and `raw-window-handle` only, and builds for `wasm32`.

---

## How

```rust
let mut renderer: Box<dyn Render> = Box::new(ExplicitRender::create(settings)?);

renderer.attach(window)?;
renderer.render_frame(RenderFrame { snapshot, terrain })?;
renderer.resize(size)?;
renderer.update_settings(settings)?;
renderer.detach()?;

renderer.destroy()?;
```

### `Render`

Three kinds of change, each with its own method and its own amount of work:

| Method | What changes | What the renderer rebuilds |
|---|---|---|
| `attach(window)` / `detach()` | Where the frame is shown | The surface and the swapchain. Nodes and resources stay |
| `resize(size)` | The window size | The swapchain and the resources sized from it. Nodes stay |
| `update_settings(settings)` | Settings: features on and off, their parameters, render scale, present mode, HDR | What the settings affect. Each feature rebuilds itself from its old state and keeps what it can |

- `render_frame(frame)` takes one frame and draws it into the window.
- `destroy()` releases everything. Call it once, when the engine shuts down.
- **The renderer is its own state.** There is no separate render state outside it.
- **Without a window** (before `attach` or after `detach`) the renderer keeps its state and `render_frame` draws nothing.
- **A window of zero size** (minimized) draws nothing until a `resize` to a non-zero size.
- **What the engine reports.** The engine reports window, size and settings changes. Surface problems such as an out-of-date swapchain stay inside the renderer.

### Types

```rust
RenderWindow { display, window, size: WindowSize }
WindowSize { width, height }
RenderSettings { present_mode, hdr, render_scale }
```

- `RenderWindow` carries the raw handles from `raw-window-handle`. The platform keeps the window alive while a renderer is attached to it.
- `RenderSettings` holds only what every renderer understands. Feature settings join when the features do. Each renderer derives its own settings from them.
- `PresentMode`: `Fifo` (vsync), `Mailbox`, `Immediate`.

### Frame

```rust
RenderFrame { snapshot: RenderSnapshot, terrain: TerrainFrame }
```

- The types come from `render_snapshot`: entities, camera, light, debug lines, terrain requests and chunks.
- Picked entity and render statistics are not part of the interface. They come back through readbacks.

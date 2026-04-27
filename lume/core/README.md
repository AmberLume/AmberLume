[← AmberLume](../../README.md) · [lume/desktop](../desktop/README.md) · [lume/android](../android/README.md)

# lume/core

Platform-agnostic application layer shared between all platform targets.

---

## What it does

`core` sits between the engine (`amber_lume`) and platform-specific launchers (`desktop`, `android`). It owns everything that is the same regardless of how the app is launched: scene setup, ECS workload composition, UI layout, and the main-loop entry point via `Lume`.

Platform launchers instantiate `Lume::create(providers, ...)` and then call `draw()` each frame — that is the entire integration contract.

---

## Contents

| Module | Purpose |
|---|---|
| `lume` | `Lume` struct — wires `AmberLume` + workloads, owns the per-frame loop |
| `scene` | `SceneManager` — loads test scenes, spawns ECS entities |
| `engine/systems` | Camera system and other app-level systems not part of the engine |
| `ui` | Shared UI layouts and widgets (built on `yakui`) |
| `tracing` | Logging initializer (one call, same setup on all platforms) |

---

## Platform contract

`core` never touches OS APIs directly. The two traits it receives from the launcher are:

- **`SurfaceProvider`** — provides raw window handles and size for Vulkan surface creation
- **`IOProvider`** — returns the list of available asset paths for resource discovery

Swapping the platform means providing a different pair of implementations. The `core` code does not change.

---

## ECS workloads

`Lume::bind_workloads` registers the `"common"` workload:

```
world_time_system
user_input_system
physics_registration_system  →  physics_iterator_system
                             →  character_physics_force_system
                             →  physics_synchronization_system
camera_system
resource_resolver_system
animation_resolver_system  →  humanoid_animation_system
                           →  animation_system
world_day_night_system
render_snapshot_system
```

Each frame `Lume::draw()` pulls input events, pushes them into the world, runs the workload, then calls `AmberLume::render()`.

---

## Controls

| Key | Action |
|---|---|
| Arrow keys | Move |
| C | Jump |

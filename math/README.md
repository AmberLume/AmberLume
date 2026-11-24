[← AmberLume](../README.md)

# math

Math types for the whole engine: `Vec2`, `Vec3`, `Vec4`, `Quat`, `Mat4`. One type per meaning, from the world down to the renderer, with no conversions between crates.

---

## Why

- **One type everywhere.** A position is a `Vec3` in the world, in physics, in the frame and in the renderer, not a `Vec3` in one crate and `[f32; 3]` in another.
- **`glam` stays inside.** The types are ours, with named fields. Operations delegate to `glam`, which no other crate sees. Replacing `glam` touches only this crate.
- **Plain data.** Every type is `#[repr(C)]`, `Copy` and `Pod`, so it goes into GPU buffers as is.

---

## How

```rust
use math::{Mat4, Quat, Vec3};

let rotation = Quat::from_axis_angle(Vec3::Y, angle);
let transform = Mat4::from_scale_rotation_translation(Vec3::ONE, rotation, position);
let forward = rotation * Vec3::Z;
```

### Types

| Type | Layout | Notes |
|---|---|---|
| `Vec2` | 8 bytes, align 4 | |
| `Vec3` | 12 bytes, align 4 | `extend(w)` gives `Vec4`, `cross` |
| `Vec4` | 16 bytes, align 16 | `truncate()` gives `Vec3` |
| `Quat` | 16 bytes, align 16 | `x, y, z, w`; `from_axis_angle`, `from_euler` / `to_euler` with `EulerOrder`, scaled axis, `slerp` |
| `Mat4` | 64 bytes, align 16 | Columns `x_axis` … `w_axis`; translation, scale-rotation-translation, `perspective_rh`, `look_at_rh`, `inverse`, `row`, transform of points and vectors |

- Vectors share component-wise operators (`+`, `-`, `*`, `/`, negation, scalar on either side) and `dot`, `length`, `distance`, `normalize`, `normalize_or_zero`, `lerp`, `min`, `max`, `clamp`, `abs`, `floor`, `ceil`, `round`, `min_element`, `max_element`.
- `from_array` and `to_array` are the bridge to libraries with their own types, such as physics.
- The set holds what the engine uses. A new operation is added here when someone needs it, delegating to `glam`.

### Cost

- In release builds (fat LTO) the conversion to `glam` and back is optimized away.
- In debug builds our crates are not optimized, so each operation is a call. `glam` called from our crates is not optimized there either, so the wrapper adds one call level.

### Adoption

New crates use `math` from the start. Old code moves to it only when touched.

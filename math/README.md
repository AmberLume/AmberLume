[← AmberLume](../README.md)

# math

Math types for the whole engine: `Vec2`, `Vec3`, `Vec4`, `Quat`, `Mat4`, plus `EulerOrder`. One type per meaning, from the world down to the renderer and the GPU, with no conversions between crates.

---

## Why

- **One type everywhere.** A position is a `Vec3` in the world, in physics, in the frame and in a push constant, not a `Vec3` in one crate and `[f32; 3]` in another.
- **`glam` stays inside.** The types are ours, with named fields. Arithmetic is component-wise in the crate, everything else goes through `glam`, which no other crate sees. Replacing `glam` touches only this crate.
- **Only what is used.** An operation is added when the first code needs it, not in advance.
- **Plain data.** The vector, quaternion and matrix types are `#[repr(C)]`, `Copy` and `Pod`, so they go into GPU buffers and push constants as is.
- **Align 4.** Shaders read buffers and push constants with the scalar layout. Every type has the alignment of `f32`, the same as `[f32; N]`, so a field keeps its offset. A struct that needs 16 asks for it with `#[repr(C, align(16))]` itself.

---

## How

```rust
use math::{EulerOrder, Mat4, Quat, Vec3};

let rotation = Quat::from_euler(EulerOrder::Yxz, pitch, yaw, 0.0);
let transform = Mat4::from_scale_rotation_translation(Vec3::ONE, rotation, position);
let forward = rotation * Vec3::Z;
```

### Types

| Type | Layout | API |
|---|---|---|
| `Vec2` | 8 bytes, align 4 | `ZERO`, `ONE`, `new`, `from_array` |
| `Vec3` | 12 bytes, align 4 | `ZERO`, `ONE`, `Y`, `Z`, `NEG_Y`, `new`, `from_array`, `to_array`, `cross`, `any_orthonormal_vector` |
| `Vec4` | 16 bytes, align 4 | `ZERO`, `W`, `new`, `from_array`, `to_array`, `truncate` |
| `Quat` | 16 bytes, align 4 | `x, y, z, w`; `IDENTITY`, `from_xyzw`, `from_array`, `from_euler`, `to_euler`, `to_scaled_axis`, `dot`, `inverse`, `normalize`, `slerp`, `*` with `Quat` and `Vec3`, negation |
| `Mat4` | 64 bytes, align 4 | Columns `x_axis` … `w_axis`; `ZERO`, `from_cols`, `from_cols_array_2d`, `from_translation`, `from_scale_rotation_translation`, `perspective_rh`, `look_at_rh`, `inverse`, `row`, `transform_point3`, `*` with `Mat4` and `Vec4` |
| `EulerOrder` | enum | `Xyz`, `Yxz` |

- Vectors share `+`, `-`, `*` and `/` with a vector or an `f32` on the right, negation, `+=`, `-=`, and `dot`, `length`, `distance`, `normalize`, `try_normalize`, `normalize_or_zero`, `lerp`.
- `from_array` and `to_array` are the bridge to arrays: asset data and API calls such as Vulkan clear values.

### Euler angles

- `from_euler(order, x, y, z)` takes the angle around each axis by its name; `order` sets only the order the rotations are applied in.
- `to_euler(order)` returns `(x, y, z)` the same way.

### Projection

- `perspective_rh` is right-handed with depth in `[0, 1]` and Y up: the clip space of D3D12, Metal and WebGPU. A renderer with another convention adapts it on its side: Vulkan flips Y, reverse-z swaps near and far.
- `look_at_rh` builds a right-handed view that looks along `-Z`.

### Cost

- In release builds (fat LTO) the conversion to `glam` and back is optimized away.
- Every function is `#[inline(always)]`, so even in debug builds the wrapper and the conversions are inlined into the caller. What stays a call in debug is the `glam` function itself, the same as when `glam` is called directly.

### Boundaries

- Crates that work with vectors, rotations and matrices use `math`. No other crate depends on `glam`.
- Libraries with their own math types are converted where they are called: `physics` converts to and from rapier types (`ToRapier`, `ToMath`), `ui` converts yakui vertices.
- Asset formats in `resource_data` keep `[f32; N]` arrays and are converted when loaded.

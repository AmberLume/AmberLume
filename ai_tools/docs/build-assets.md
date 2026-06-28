# build-assets

Run the AmberLume **asset/shader builder** and (optionally) locate a generated
symbol.

## Why

The builder is a standalone binary (`cargo run -p builder`) — it is **not** a
`build.rs` and is **not** run automatically by `cargo build`. The only `build.rs`
in the repo just creates an empty `target/generated/resources.rs` so the engine
still compiles. Consequence: after you add, rename, or edit a shader/asset, the
generated manifest and the packed `*.alpaca` archives are **stale** until you run
the builder by hand. This script is that step, plus a shortcut to read back the
generated constant names.

## Usage

```bash
ai_tools/build-assets.sh                 # build everything
ai_tools/build-assets.sh occlusion       # build, then grep the manifest for "occlusion"
```

## What it does

1. `cd` to the workspace root and run `cargo run -p builder`.
2. Print the path to the regenerated manifest `target/generated/resources.rs`.
3. If a pattern is given, `grep -ni` the manifest for it (to find the generated
   constant name).

## What the builder generates

| Output | Path | Use |
|--------|------|-----|
| Typed manifest | `target/generated/resources.rs` | **Read this** to find a symbol, e.g. `shaders::MAIN_FRAG`, `meshes::BASIC`. |
| SPIR-V / intermediates | `target/generated/alpaca/**` | Compiled shader/asset blobs. |
| Packed archives | `target/distribution/assets/*.alpaca` | What the engine loads at runtime. |
| Incremental cache | `target/generated/.builder_cache.bin` | blake3 content hashes + `#include` deps. |

Constant naming: file stem, uppercased, non-alphanumerics → `_`, duplicates
suffixed `_0/_1`. Example: `occlusion_cull.comp` → `shaders::OCCLUSION_CULL_COMP`.

## Interpreting the result

- **Exit 0** + a final `Cache: untouched=.. touched=.. new=.. (N orphan removed)`
  line = success.
- The builder logs at `builder=trace` (INFO lines like `Compiling shader…`,
  `Cached shader…`, `Writing data into…`). `RUST_LOG` is ignored.
- **A shader with a syntax error panics the worker** (the processor uses
  `.expect()`), so a non-zero exit with a Rust panic backtrace usually points at
  a bad GLSL file rather than a tooling bug.

## Inputs

- GLSL shaders: `lume/core/resources/shaders/**` (`.vert` / `.frag` / `.comp`;
  `.glsl` files are include-only and are not compiled directly).
- glTF assets: `target/generated/prebuild/assets/**` (populated by the Blender
  export step, `blender/export_blender_assets.sh`).

## Related files

- `builder/src/main.rs` — entry point.
- `builder/src/manifest.rs` — generates `resources.rs`.
- `builder/src/processors/shader_processor.rs` — GLSL → SPIR-V via shaderc.
- `amber_lume/src/resources/resource_manifest.rs` — `include!`s the manifest.

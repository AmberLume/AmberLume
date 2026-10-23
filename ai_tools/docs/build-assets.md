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
| Typed manifest | `target/generated/resources.rs` | **Read this** to find a symbol, e.g. `shaders::MAIN`, `meshes::BASIC`. |
| SPIR-V / intermediates | `target/generated/alpaca/**` | Compiled shader/asset blobs. |
| Packed archives | `target/distribution/assets/*.alpaca` | What the engine loads at runtime. |
| Incremental cache | `target/generated/.builder_cache.bin` | blake3 content hashes + shader `import` deps (from `slangc -depfile`). |

Constant naming: file stem, uppercased, non-alphanumerics → `_`, duplicates
suffixed `_0/_1`. Example: `gtao_depth/gtao_depth.shader` → `shaders::GTAO_DEPTH`.

## Interpreting the result

- **Exit 0** + a final `Cache: untouched=.. touched=.. new=.. (N orphan removed)`
  line + no `Error while dispatching task!` lines = success.
- The builder logs at `builder=trace` (INFO lines like `Compiling shader…`,
  `Cached shader…`, `Writing data into…`). `RUST_LOG` is ignored.
- **A shader with a compile error does not fail the build**: `slangc` prints the
  diagnostic (file, line, `error[E…]`) to stderr, the builder logs
  `Error while dispatching task! slangc failed with exit status …`, and the exit
  code stays 0. Check the output for those lines after editing a shader.

## Inputs

- Slang shaders: `lume/core/resources/shaders/**`. A program is a `.shader` file
  holding every stage of one pipeline (entry points marked `[shader("...")]`);
  `.slang` files are modules pulled in with `import` and are not compiled
  directly. Requires `slangc` in `PATH`.
- glTF assets: `target/generated/prebuild/assets/**` (populated by the Blender
  export step, `blender/export_blender_assets.sh`).

## Related files

- `builder/src/main.rs` — entry point.
- `builder/src/manifest.rs` — generates `resources.rs`.
- `builder/src/processors/shader/shader_processor.rs` — Slang → SPIR-V via the `slangc` CLI.
- `amber_lume/src/resources/resource_manifest.rs` — `include!`s the manifest.

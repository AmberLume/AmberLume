# AmberLume – Agent Instructions

This document defines how an LLM agent should understand, navigate, and assist with the **AmberLume** project.  
It describes the architecture, modules, conventions, and review policy.

---

## 1. Project Overview

**AmberLume** is a **multi-module Rust project** that includes:
- A pre-build asset compiler and packer,
- A cross-platform game engine core,
- And an example application using the engine.

The project focuses on **low-level control**, **performance**, and **clean, independent design**.  
It uses **Vulkan via Ash**, a **custom binary asset format (.alpaca)**, and minimal external dependencies.

Key principles:
- Optimize everything possible at build time.
- Minimize abstraction, maximize control.
- Keep the engine platform-agnostic.
- Maintain code clarity and explicit logic.

---

## 2. Modules

### `alpaca`
Pre-build resource preparation and packaging module.

#### Responsibilities:
1. **Scan** the `assets/` directory and discover source files.
2. **Compile and optimize** them into intermediate formats (e.g. `png → ktx2`, `glb → separated meshes and textures`, `shader → spv`) and store results in `generated/assets/`.
3. **Pack** the optimized files from `generated/assets/` into a single `.alpaca` archive inside `generated/`, creating an index and offsets for fast access.

The result is an independent resource pack that can be read in one operation instead of traversing multiple files.

#### Internal structure:
- `walker` — file scanning and collection
- `compiler` — resource optimization and transformation
- `packer` — packaging and indexing
- `unpacker` — unpacking and indexing
- `cli` — command-line interface for automation

---

### `amber_lume`
The core **engine** module.

#### Responsibilities:
- Vulkan rendering via Ash
- Pipeline, mesh, material, texture, and camera managers
- ECS and basic systems
- Memory and resource management **via platform-provided interfaces** (no direct file or `mmap` access)
- GPU-scene architecture and caching layer

#### Principles:
- Fully platform-independent
- Compact, fast, and self-contained
- Provides a clean, explicit API for applications

---

### `lume`
Example game / test application using the engine.

#### Responsibilities:
- Window management (`winit`)
- Engine initialization and runtime loop
- File and input handling
- Integration of platform-specific systems (including file I/O and `mmap`)

---

## 3. Architectural Priorities
- Rust 2024 edition
- Safety and performance over universality
- Minimal heap allocations at runtime
- Use of LRU caches and static GPU buffers
- No global singletons
- Data and resources are independent entities, not scene-bound

---

## 4. Coding Conventions
- Functions and variables — `snake_case`
- Structs and types — `PascalCase`
- Module names — short and clear
- Public API must be self-descriptive
- Prefer `Arc` over `Rc`
- `unsafe` only for Vulkan FFI or justified performance reasons

---

## 5. Review Policy
When performing a review, the agent should:

1. **Start with a summary** — what changed, why, and whether it supports project goals.
2. **List all issues** found in modified code only:
    - Typos
    - Inefficient data types
    - Leaks
    - Repetition
    - Wrong API use
    - Logic deviations

The agent must analyze **all modified code** and report **every issue found**, not stop after the first problem.

3. For each issue, describe:
    - **Problem:** what is wrong
    - **Consequence:** why it matters
    - **Suggestion:** how to fix it

Do **not** comment on untouched code unless it directly affects the modified logic.  
The agent must analyze **all modified code** and report **every issue found**, not stop after the first problem.

---

## 6. Context
- Target: cross-platform game engine and toolkit
- Platforms: Windows / Linux (Android planned)
- Technologies:
    - **Ash (Vulkan)**
    - **Rapier (Physics)**
    - **KTX2 / BasisU / Meshopt** for asset optimization
    - **mmap** for memory mapping (handled by platform layer)
- Build: Cargo workspace with multiple crates

---

## 7. Response Style
- Be concise and direct.
- No unnecessary commentary or emotional tone.
- When code is requested — output only code.
- When reviewing — follow this structure:
    1. Summary
    2. Issues
    3. Suggestions

---

## 8. Agent Tooling (`ai_tools/`)

The repository root contains an **`ai_tools/`** directory with helper scripts and
documentation written specifically for AI agents working on this project.
**Before re-deriving how to build, run, or validate the project, read
[`ai_tools/README.md`](ai_tools/README.md).**

Available tools (each has a guide in `ai_tools/docs/`):
- **`build-assets.sh`** — run the asset/shader builder (`cargo run -p builder`)
  and locate generated symbols in `target/generated/resources.rs`.
- **`smoke-run.sh`** — build and launch the `desktop` app for a few seconds
  (auto-closed via `timeout`), capture logs, and report whether it started and
  rendered.
- **`vk-run.sh`** — the same launch with all Vulkan validation enabled, then
  scan the log for validation findings.

When adding a new agent tool, register it in `ai_tools/README.md` and add a
matching `ai_tools/docs/<tool>.md` guide.

---

*End of `agents.md`*
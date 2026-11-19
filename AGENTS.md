# retro-blit

Retro-blit is a Rust library for DOS-style games using low-resolution, 8-bit indexed-color framebuffers and a 256-color RGB palette. The root package uses Rust 2018; the larger demos are separate Rust 2021 packages.

## Repository layout

- `src/window/`: window/event loop, input, `RetroBlitContext`, palette management, and presentation.
- `src/rendering/`: indexed-color surfaces, blitting, rasterization, tessellation, transforms, and fonts.
- `src/audio/`: Rodio-backed `SoundDriver` and decoded `SoundHandle` values.
- `src/format_loaders/`: IM256 and 256-color BMP loaders.
- `src/math_utils/`: geometry, collision, and BSP helpers.
- `examples/`: small examples belonging to the root Cargo package.
- `asteroids-example/`, `dungeon-crawler-example/`, `map-editor-example/`, `portal-map-viewer/`: independent Cargo packages that depend on the root library by path.
- `aseprite/extension/`: Lua source for the IM256 Aseprite exporter; edit this source rather than the packaged `.aseprite-extension` file.

## Build and verification

There is no shared Cargo workspace. Run commands in the package being changed.

Root library and small examples:

```sh
cargo build
cargo test
cargo build --examples
cargo run --example test_window
```

Larger demos, using asteroids as an example:

```sh
cd asteroids-example
cargo build
cargo run
```

When changing the root library, run the root tests and build each affected dependent demo from its own directory. Each package has its own lockfile and target directory.

The test suite currently contains no unit tests, so a passing `cargo test` is only a compile/doc-test check. Graphical and audio changes also need runtime verification; compilation alone does not prove rendering or audible playback works.

Keep `.cargo/config.toml`'s `CMAKE_POLICY_VERSION_MINIMUM=3.5` setting: it allows the transitive SDL2 build to configure with current CMake versions.

## Project conventions

- Preserve the indexed-color model: pixel buffers contain palette indices, not RGB/RGBA pixels.
- Keep changes compatible with the existing `ContextHandler`, `RetroBlitContext`, surface/blitting, and builder-style APIs used by the examples.
- Assets compiled into binaries generally use `include_bytes!` with paths relative to the source file.
- Root API changes can affect all four independent demo packages; check their usages before changing public types or signatures.
- Match the Rust edition and dependency style of the package being edited. Avoid unrelated formatting or dependency upgrades.
- Audio uses Rodio 0.23. The output buffer is explicitly 1024 frames because larger requested buffers have failed on CoreAudio devices; verify device initialization and playback if changing this path.

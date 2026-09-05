# Pointer rendering fork

Based on EVP v0.19.0 (`cb96c0aaffc1342a52ebf085b6e5567cfa938031`).

This fork changes presentation only:

- Original macOS-style black arrow with a fine white outline; no Apple assets.
- Shared geometry for SVG and antialiased GIF/PNG output.
- Small neutral click/drag feedback instead of saturated circles.
- Input coordinates, modifier dispatch and terminal recording remain unchanged.

`src/pointer.rs` owns the geometry and raster coverage. The existing raster
and SVG renderers consume it. There are no new dependencies or tape settings.

Build with the upstream macOS libghostty prebuild procedure, then:

```sh
cargo build --locked --release --target aarch64-apple-darwin --bin evp
cargo test --locked --release --target aarch64-apple-darwin --lib
```

Zig 0.15.2 cannot link its build runner against recent macOS 26 SDK stubs.
Use a compatible SDK for that bootstrap; do not change global Xcode selection
just for this build. Ensure the generated libghostty pkg-config prefix is
`${pcfiledir}/../..`, as in upstream release CI.

# Pointer rendering fork

Based on EVP v0.19.0 (`cb96c0aaffc1342a52ebf085b6e5567cfa938031`).

This fork changes presentation only:

- Original macOS-style black arrow with a fine white outline; no Apple assets.
- Shared geometry for SVG and antialiased GIF/PNG output.
- Neutral click/drag feedback instead of saturated circles.
- Arrow compresses to 82% while pressed and returns to full size on release;
  the hotspot stays fixed in both raster and SVG output.
- MouseMove and MouseDrag apply easing to distance along the spline over elapsed
  time, not to event timestamps. Linear means constant path speed. Intermediate
  rendering uses linear interpolation rather than applying easing a second time.
- Back/elastic progress is clamped to the path endpoints; timestamps remain
  ordered. Stationary paths and zero-duration movement retain exact endpoints.
- Tape syntax, modifiers and configured durations are unchanged. Nonlinear motion
  intentionally differs from upstream's inverted easing. This correction is not
  included in the earlier pointer-v0.19.0-1 binary.

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

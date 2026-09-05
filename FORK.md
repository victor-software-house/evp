# Pointer rendering fork

Based on EVP v0.19.0 (`cb96c0aaffc1342a52ebf085b6e5567cfa938031`).

This fork changes presentation only:

- Original macOS-style black arrow with a fine white outline; no Apple assets.
- Shared geometry for SVG and antialiased GIF/PNG output.
- Neutral click/drag feedback instead of saturated circles.
- Arrow compresses to 82% while pressed and returns to full size on release;
  the hotspot stays fixed in both raster and SVG output.
- Scripted pointer paths hold the speed their tape asks for: spline spans are
  respaced by arc length before the sample times are laid out, and a segment
  without its own easing interpolates linearly. Upstream sampled each
  Catmull-Rom span by curve parameter, which with duplicated end control points
  reduces a straight two-point move to 0.5u + 1.5u^2 - u^3 and swings its speed
  2.5x between the span ends and its middle. Across a multi-point move that
  reads as a sawtooth, and the faster the move the worse it looks.
  Gestures captured by `record` set their easing explicitly and are unaffected.
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

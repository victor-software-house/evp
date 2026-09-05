# EVP fork contributor guidance

Read README.md for the supported install and capture workflow, FORK.md for the
patch scope, and ARCHITECTURE.md before changing the renderer or PTY runner.

## Scope

This is a small downstream fork of EVP v0.19.0. Keep changes focused on real
terminal capture and pointer presentation. Do not introduce a second capture
engine, install-time compilation or unsolicited infrastructure.

- Rust source is under src/; the CLI is src/bin/evp.rs.
- src/pointer.rs owns original pointer geometry shared by GIF/PNG and SVG.
- src/render_gif.rs and src/render_svg.rs own their output formats.
- src/script/ owns tape parsing; src/runner.rs drives execution.
- skills/evp/SKILL.md is the bundled operator skill.

## Runtime invariants

The PTY runner must not block on rendering. Consumer channels use try_send and
bounded queues. Drop every sender clone before joining renderer workers.
libghostty values stay on the runner thread; only owned frame data crosses it.
Font loading and fallback belong in src/font.rs. SVG fonts must be subsetted.
Keep input dispatch and modifier semantics unchanged by cosmetic pointer work.

Record real execution. Demo tests own isolated files, homes and sessions. Never
restart or send input into a user's existing shell/session for verification.
Do not hide or clear restored scrollback to improve a screenshot.

## Build and verification

Normal installation uses prebuilt archives. Build only on the operator-approved
builder; do not start a local build when remote-only execution was requested.
The existing .cargo/config.toml defaults to Linux musl. For Apple Silicon,
explicitly pass --target aarch64-apple-darwin.

Prebuild libghostty using the pinned source/Zig procedure in the upstream
release workflow. Set PKG_CONFIG_PATH to its share/pkgconfig directory. Correct
the generated .pc prefix to `${pcfiledir}/../..`; do not change dependencies or
system-wide SDK selection to bypass a local build failure. Recent macOS SDKs
need special care with Zig 0.15.2; see FORK.md.

```sh
cargo fmt --check
cargo build --locked --release --target aarch64-apple-darwin --bin evp
cargo test --locked --release --target aarch64-apple-darwin --lib
```

Use release builds for capture/performance verification. Debug rendering is not
representative. Check actual GIF timing, SVG output, PNG and interactive input
behavior, not only parser acceptance. Add focused regressions for changed
geometry, events or formatting. Use exact targeted edits in large renderers.

## Distribution contract

Each release archive contains evp, README.md, AGENTS.md, FORK.md, ARCHITECTURE.md,
LICENSE, licenses/, skills/evp/ and examples/quickstart.tape. Publish its SHA-256
sidecar and install.sh. Verify a fresh download and isolated installation.
Names follow evp-<tag>-aarch64-apple-darwin.tar.gz.

Fork tags use pointer-v<upstream-version>-<revision>. They do not trigger the
inherited upstream v*.*.* release workflow. Do not run that workflow for fork
releases or spend GitHub-hosted runner minutes; use an approved builder.

Update README and the bundled skill with user-visible CLI/capture changes.
Keep all known output formats aligned. Do not publish uncommitted working-tree
binaries. Tag the exact verified revision and do not overwrite release assets.

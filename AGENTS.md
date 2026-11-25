# EVP fork contributor guidance

Read README.md for the supported install and capture workflow, PATCHES.md for
the patch stack, and ARCHITECTURE.md before changing the renderer or PTY runner.

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

## Patches

The `downstream` branch is a forkctl patch stack on upstream v0.19.0.
Change it only through `mise run fork`: `patch create` or `patch select`,
stage, `patch refresh`, `patch finish`, then `publish`. patches/fork.yaml
states each patch's purpose, scope and drop condition; PATCHES.md and
patches/downstream/ are generated from it.

## Build and verification

Normal installation uses released archives, which GitHub Actions builds
(.github/workflows/fork-release.yml). Do not build release binaries locally.
`mise run evp:build aarch64-apple-darwin` is the one build: it
prebuilds libghostty-vt from the pinned Ghostty commit with Zig 0.15.2, builds
evp for that target, and stages the archive files. .cargo/config.toml defaults
to Linux musl, so a cargo command names its target. Zig 0.15.2 cannot link its
build runner against macOS 26 SDK stubs, so the release job selects Xcode 16.4.

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

Each release archive contains evp, README.md, AGENTS.md, PATCHES.md,
ARCHITECTURE.md, LICENSE, licenses/, skills/evp/ and examples/quickstart.tape
at its root, as evp-<version>-aarch64-apple-darwin.tar.gz with its .sha256.
verctl publishes it as the vendored set `evp` in .ctl/ver.yaml.

The version is upstream's version plus -vsh.N, such as 0.19.0-vsh.1, and the
tag is evp-<version>. It never matches the inherited upstream v*.*.* release
workflow, so do not run that workflow. Raise N in .ctl/ver.yaml for a new build
on the same upstream base, and start again at 1 after a rebase; the push to
downstream that changes it releases.

Update README and the bundled skill with user-visible CLI/capture changes.
Keep all known output formats aligned. Do not publish uncommitted working-tree
binaries. Tag the exact verified revision and do not overwrite release assets.

# EVP — terminal recordings with a natural pointer

Run real terminal programs and capture GIFs, animated SVGs and PNG screenshots.
This fork adds a crisp macOS-style pointer with visible press/release feedback.
It is based on [EVP v0.19.0](https://github.com/HalFrgrd/evp/tree/v0.19.0).

## Install without building

**Current binary: macOS on Apple Silicon (arm64).** Other platforms are not
packaged by this fork yet. No Rust, Zig, Bun or runtime dependencies required.

With [mise](https://mise.jdx.dev/):

```sh
mise use -g github:victor-software-house/evp@pointer-v0.19.0-1
evp --version
```

Or download the installer, inspect it, then run it:

```sh
curl -fL https://github.com/victor-software-house/evp/releases/download/pointer-v0.19.0-1/install.sh -o /tmp/evp-install.sh
sh /tmp/evp-install.sh
```

The installer checks the archive's SHA-256 before installing. It defaults to
`~/.local/bin/evp` and `~/.local/share/evp/`. It does not change your PATH or
shell configuration. Set `EVP_INSTALL_DIR` to select another binary directory.
The documentation lives at `../share/evp` relative to that directory.
Set `EVP_VERSION` to select an explicit fork release.

Release archives include the executable, checksums, license, documentation,
a working example and the [bundled skill](skills/evp/SKILL.md).
For manual installation, download the archive and its `.sha256` from
[Releases](https://github.com/victor-software-house/evp/releases), run
`shasum -a 256 -c <archive>.sha256`, extract, and put `evp` on PATH.

## Your first capture

Create `hello.tape`:

<!-- START_REF_SCRIPT -->
```text
Output hello.gif
Output hello.svg
Set Shell /bin/zsh -f
Set Theme "Catppuccin Mocha"
Set Cols 72
Set Rows 12
Set FontSize 20
Set Padding 24
Set Framerate 50
Env PS1 "❯ "
Sleep 500ms
Type "printf 'Hello from EVP\\n'"
Enter
Sleep 1s
Screenshot hello.png
```
<!-- END_REF_SCRIPT -->

Then:

```sh
evp validate hello.tape
evp hello.tape
open hello.gif
```

If your image viewer does not animate GIFs, open the file in a browser.
The archive contains this example at `examples/quickstart.tape`.

## Record an interactive session

```sh
evp record --output demo.tape --output demo.gif --shell zsh --cols 90 --rows 24
```

Exit the recorded subshell when finished. Edit the resulting tape and replay it
with `evp demo.tape`. Tape commands execute real programs: review downloaded tapes
before running them. Isolate demo files and environments; never record secrets.

## Mouse-driven demos

Coordinates are zero-based terminal columns and rows, not pixels. For a target
at column 12, row 5, a starting point is:

```text
MouseMove@300ms@Linear 24 8 12 5
Sleep 250ms
Ctrl+Click@120ms 12 5
Sleep 350ms
MouseMove@450ms@Linear 12 5 24 8
```

The sleeps provide pre-click hover and post-release dwell. `Click@120ms` holds
the button for that duration; it is not an extra delay after a click. Omit
`Ctrl+` when the application expects an ordinary click. Inspect the actual app:
these timings are a starting point, not proof of natural interaction.

GIF timing uses hundredths of a second. 50 fps maps to 20 ms motion frames.
Static frames may be combined into longer holds. Check the exported animation
in a browser; do not infer its smoothness solely from `Set Framerate`.

## Themes, fonts and synchronization

- `evp themes` lists presets. `Set Theme { "background": "#232136", "foreground": "#e0def4" }` accepts custom colors.
- Use the bundled font by omitting a font setting. `Set Font` is **not valid**
  in v0.19.0. `Set FontFamily "/absolute/path/font.ttf"` selects a font file;
  installed family names are not reliably resolved.
- `Wait+Screen@10s /ready/` waits for rendered text. Prefer it to guessed startup sleeps.
- `Screenshot frame.png` captures a real rendered frame. SVG and JSON are also supported.
- `evp --help` and `evp record --help` describe CLI flags.
- `print-ref-script` prints a commented quickstart plus upstream's test tape.
  The bundled `examples/quickstart.tape` is the minimal runnable starting point.

## Agent skill

The archive's `skills/evp/SKILL.md` teaches capture, isolation, mouse choreography
and verification. With the installer it is at `~/.local/share/evp/skills/evp/`;
with mise it is inside `mise where github:victor-software-house/evp@pointer-v0.19.0-1`.
Point your agent's skill loader there or copy that directory into its supported
project skill directory. Installation does not alter your agent configuration.

## Development

Proposed work: [natural click interactions](docs/plans/click-interactions.md)
(plan only; not implemented in the current release).

See [AGENTS.md](AGENTS.md), [FORK.md](FORK.md) and the upstream
[architecture](ARCHITECTURE.md). Normal users should install the binary.
The fork remains experimental; pin a release. A new demo or unreleased branch
change does not silently update an installed binary.

MIT. Upstream copyright and bundled font notices are retained in `LICENSE`
and `licenses/`. The pointer is original macOS-style artwork, not an Apple asset.

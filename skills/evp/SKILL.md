---
name: evp
description: Create real terminal GIFs, animated SVGs and PNG screenshots with EVP. Use for terminal demos, CLI walkthroughs, TUI recordings, mouse-driven promotional captures, tape scripting and exported-animation verification.
license: MIT
compatibility: EVP on PATH; this fork publishes an Apple Silicon macOS binary. No build toolchain is required for capture.
---

# EVP terminal capture

Use the installed binary, not a source build. `evp --help`, `evp record --help`
and `evp themes` are the discovery commands. A `.tape` launches real programs;
inspect it before execution. Do not record private paths, tokens or unrelated
sessions. Use disposable homes, files and sessions for integration demos.

## Capture workflow

1. Define the actual interaction and visible completion condition.
2. Fix terminal geometry and choose the requested theme. Inspect the user's
   current theme when asked to match it rather than guessing a preset.
3. Run a tape against the real application. Prefer `Wait+Screen@10s /ready/`
   to startup sleeps. Use `Screenshot before.png` and `Screenshot after.png`.
4. Inspect both frames and animation in a browser. Verify the action reached the
   application; a moving pointer is not proof of a successful click.
5. Deliver the GIF and still, with the source tape and exact recorder version.
   Clearly label synthetic demo data; do not imply a fixture is real agent work.

## Minimal tape

```text
Output demo.gif
Output demo.svg
Set Shell /bin/zsh -f
Set Cols 80
Set Rows 16
Set FontSize 20
Set Padding 24
Set Framerate 50
Set Theme "Catppuccin Mocha"
Env PS1 "❯ "
Sleep 500ms
Type "printf 'Hello from EVP\\n'"
Enter
Sleep 1s
Screenshot demo.png
```

Run `evp validate demo.tape`, then `evp demo.tape`. On macOS, open the GIF in a
browser when the default viewer only shows still frames.

## Mouse choreography

Coordinates are zero-based cells. Derive the target from a captured frame or
application geometry, not a guess. Use a short approach, pre-click hover, button
press/release, post-release dwell, and deliberate withdrawal. Do not immediately
fly away or leave the pointer covering the result.

```text
MouseMove@300ms@Linear 24 8 12 5
Sleep 250ms
Ctrl+Click@120ms 12 5
Sleep 350ms
MouseMove@450ms@Linear 12 5 24 8
```

`Click@120ms` is a 120 ms held press. Use Ctrl only when the target requires it.
This fork compresses the arrow while pressed and adds neutral click feedback.
Treat these durations as a starting point; inspect actual motion. Avoid long
cinematic easing for ordinary clicks.

The native easing correction after pointer-v0.19.0-1 makes EaseInOutCubic map
time to path distance: slow start/end, faster middle. Linear is constant-speed.
Use one eased movement instead of hand-timed linear legs on corrected builds.
Back/elastic progress is bounded by path endpoints. Check the recorder revision:
the first pointer binary predates this correction.

For GIF, 50 fps gives an exact 20 ms cadence. Still frames can be coalesced;
verify distinct movement frames and playback, not the average frame count over
long holds. With ImageMagick installed, inspect delays using
`magick identify -format '%T\n' demo.gif` (hundredths of a second).

## Pitfalls

- Omit font settings to use bundled fonts. `Set Font` is invalid in v0.19.0;
  `FontFamily` expects a usable font file, not a reliably resolved family name.
- `print-ref-script` includes a commented quickstart and upstream's test tape;
  use the bundled `examples/quickstart.tape` for a minimal runnable start.
- Preserve history and typed input. Never fake a prompt or send hidden Enter
  merely to improve presentation. Fix ordering in the application if needed.
- Test full-screen/alternate-buffer apps and modifier-click dispatch explicitly.
- Build only when changing EVP itself, on the approved builder. Install and
  capture do not require Rust, Zig, Bun, a browser engine or ffmpeg.

The release archive also carries README.md, AGENTS.md and FORK.md for install,
contributor and downstream-contract details.

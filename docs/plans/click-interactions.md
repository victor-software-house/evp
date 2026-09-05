# Natural click interactions

Status: proposed plan only. No implementation or build is authorized by this
change. Public API and recording-format details below require approval before
implementation.

## Outcome

Make a natural click easy to author and make every ordinary click visibly read
as a press followed by a release. Keep actual input, recorded evidence and all
rendered formats synchronized. Reuse EVP's scheduler, renderer and easing
library; do not add another automation or animation framework.

### User decisions

- Approach, hover before pressing, remain on target during the click, and dwell
  after release. Do not immediately flee the target.
- Motion must look smooth at at least 30 fps; verify the exported motion, not
  merely a framerate setting. Use 50 fps for GIF's exact 20 ms cadence.
- Keep the macOS-style pointer, subtle feedback and requested terminal palette.
- Preserve actual terminal execution, scrollback and typed input.
- Builds and tests requiring compilation run only on the approved remote
  builder, currently macbook-portable. Never build on the capture workstation.
- Keep Claude's active demo work isolated. Do not edit its tape or replace its
  binary during this work.
- Write and push this plan first. Do not implement it yet.

### Non-goals

No desktop automation, semantic widget detection, image matching, AI movement,
randomized human imitation, new capture engine, automatic withdrawal, extra
runtime dependency or general-purpose animation DSL. Do not change zsh prompt
behavior, Herdr history restoration or the plugin's resume implementation.

## Existing implementation and prerequisite

Baseline: released fork `pointer-v0.19.0-1`, commit `b35332e`.

- `src/script/ast.rs` already models Click, RightClick, DoubleClick, MouseMove,
  MouseDrag and low-level MouseInput.
- `src/runner.rs::build_timeline` schedules the actual press/release edges;
  Click's duration is a button hold, not an after-click pause.
- `MouseSegment` and `resolve_mouse_position` produce pointer state.
- `src/pointer.rs` shares arrow geometry. GIF/PNG and SVG currently switch
  compression/halo by Moving/Clicking/Dragging state rather than elapsed phase.
- `RawFrame` and stored Key/Diff frames carry only position and MouseState.
  `RawFrame::is_visually_identical` can coalesce unchanged frames.
- SVG's MouseSpan aggregates sampled position/state intervals.

Integrate and verify the native movement correction first:
[`b9b69f8`](https://github.com/victor-software-house/evp/commit/b9b69f8).
It fixes inverted timing, uneven Linear motion, negative-time easing failures,
and repeated easing between samples. Do not preserve hand-timed linear legs as
an alternative implementation after that correction is available.

## Proposed design

### 1. A small interaction helper

Provide one library helper that expands an explicit click interaction into the
existing movement, sleep and click events. A thin tape command calls the same
helper. Do not introduce a second scheduler or execute hidden input.

Proposed public shape (names are provisional):

- Rust: `ClickInteraction` with target, optional origin, modifiers, approach
  duration/easing, hover duration, press duration and post-release dwell.
- Tape: `ClickAt <col> <row>` with optional named settings such as
  `from=24,8 approach=300ms hover=250ms press=100ms dwell=300ms`.
  Modifier prefix remains consistent with existing commands: `Ctrl+ClickAt`.
- Explicit origin wins; otherwise use the known pointer position at that point
  in the timeline. If neither exists, fail validation with an actionable error.
  Never invent an off-screen origin or silently teleport.
- Same-position clicks omit approach but retain hover, press and dwell.
- One deterministic expansion:
  `MouseMove → Sleep(hover) → Click(press) → Sleep(dwell)`.
- Withdrawal remains an explicit subsequent MouseMove. Not every click should
  move away, and the helper must not hide what the author intends.

Starting defaults to evaluate, not claims about universal human timing:
300 ms approach using corrected EaseInOutCubic, 250 ms hover, 100 ms press,
300 ms post-release dwell. Every phase is independently configurable, including
zero-duration hover/dwell. Reject negative durations, invalid coordinates,
unknown options and unsupported modifier combinations at the parser boundary.

Existing `Click`, `RightClick`, `DoubleClick` and `MouseMove` syntax and input
semantics do not change. The convenience helper initially supports a single
left click plus existing modifiers; do not grow a preset catalogue.

### 2. Time-based feedback for all click entry points

Visual feedback is separate from input scheduling. It must not add an input
hold, duplicate a click or change when the application receives release.

Use a shared, pure phase evaluator driven by exact scheduler/recorded input
edges and the frame timestamp:

| Phase | Presentation | Input |
| --- | --- | --- |
| Approach/hover | Full-size pointer; no halo | Movement only |
| Press onset | Brief compression toward pressed scale | One press edge |
| Held | Stable pressed scale; restrained halo | No repeated press |
| Release | Return to full size; short fading/expanding halo | One release edge |
| Dwell | Full-size pointer remains at target | None |

Suggested initial visual parameters: 0.88 pressed scale, 60 ms compression,
100 ms recovery and 180 ms release halo. Evaluate these visually before fixing
them as defaults. Start release recovery from the actual current scale when a
hold is shorter than compression; no jump to a never-reached scale. A short
click must still have discernible release feedback. No elastic bounce by
default, oversized disk, permanent halo or pointer disappearance after release.

The arrow scales around the hotspot. The release halo stays anchored at the
actual click location if the pointer subsequently moves; it must not follow the
cursor across the result. A new click replaces the prior transient feedback
rather than allocating an unbounded effect queue. Double-clicks preserve their
two input edges and two distinct feedback onsets. A drag must not emit a click
pulse at every motion sample; retain drag semantics and show release once.

### 3. One timeline across live capture and re-rendering

Do not infer exact click timing solely from sampled MouseState changes: a press
and release can both fall between frame deadlines. Preserve input-edge timing
through the recording path so short clicks and replay are correct.

Proposed representation to approve before implementation: an optional,
bounded pointer-feedback snapshot on RawFrame and stored Key/Diff frames,
containing the active press/release timestamps and click origin. Keep pointer
position and MouseState as input facts; do not encode animation scale as a
fake mouse coordinate. A release snapshot survives until its effect expires.

Update every writer/reader together: runner, interactive recorder, full-recording
consumer, recording builder/reconstruction, JSON export/import, GIF/PNG and SVG.
Decide the exact field names and document the JSON contract before editing.
New exports must preserve enough information for equivalent offline rendering.

The current upstream EVP v0.19.0 JSON producer remains a named external source
of recordings without exact edge metadata. If retaining import support, define
and test its limited behavior explicitly: inferred frame-boundary timing, with
no claim of recovering sub-frame clicks. Do not scatter fallback logic across
renderers. Inventory existing JSON fixtures/readers before approving this
compatibility decision; otherwise make a documented format break and reject
old data clearly. This is a decision gate, not permission to silently change
stored data.

Compute animated scale/halo from the same shared evaluator in every renderer.
A standalone PNG at time t must match the GIF frame and SVG presentation at t.
Keep elapsed-time calculations independent of frame count and wall-clock speed.

Animation-only changes must survive equality checks, dirty-region detection
and frame coalescing. Include both the previous and current arrow/halo bounds
when restoring pixels, including an anchored halo after pointer withdrawal.
Handle the final frame/tail explicitly: let a final release effect finish
without delaying or inventing application input.

## Ordered implementation and proof

1. **Approve contracts and integrate movement prerequisite.**
   Confirm helper naming, defaults and recording representation/compatibility.
   Inventory current readers, writers and fixtures. Merge the native movement
   correction without changing Claude's working checkout.
   → Proof: existing motion regressions and package checks pass remotely.

2. **Add failing timing and presentation regressions.**
   Characterize existing Click input edges; reproduce missing sub-frame click
   feedback and state-only visual jumps.
   → Proof: new targeted tests fail for those reasons on the baseline, not from
   unrelated SDK/build setup.

3. **Implement the single helper expansion.**
   Cover parser, Rust API export and tape serialization together. Preserve
   modifiers, coordinates and press duration exactly.
   → Proof: helper and manually expanded events have identical input timelines;
   zero/same-position phases, missing origin and malformed options are tested.

4. **Implement exact feedback timing through recording and renderers.**
   Wire both scripted and interactive input into the shared phase evaluator;
   update all serialization and rendering consumers in one complete change.
   → Proof: ordinary/right/double/drag clicks, very short holds, repeated clicks,
   nonzero recording start time, Wait delays, Hide/Show, cursor disappearance,
   zero-duration events and recording replay have deterministic outcomes.
   Verify exact input-edge counts and no invented events.

5. **Verify rendered behavior, not only math.**
   Run a small isolated mouse-reporting fixture and a real Herdr Ctrl-click
   interaction. Capture pre-hover, pressed, released, dwell and withdrawal
   frames. Compare GIF, SVG and PNG at shared timestamps. Check a click with no
   terminal text changes so frame coalescing cannot hide missing animation.
   → Proof: 50 fps motion/feedback frames use 20 ms GIF delays; a 30 fps setting
   has no unintended gaps beyond its cadence/format quantization. Intentional
   stationary holds are measured separately. Input logs match the visual click.
   Inspect the final animation in a browser and get operator visual approval.

6. **Document and distribute the verified behavior.**
   Update README's build-time reference markers, quickstart, FORK.md, AGENTS.md
   where needed, and skills/evp/SKILL.md. Remove obsolete state-only drawing and
   any demo compensation made unnecessary by the native path; coordinate demo
   edits with its owner rather than overwriting their work.
   → Proof: tests pass, packaged quickstart renders, bundled skill validates,
   archive contains docs/skill, and a fresh checksum-verified binary install
   works without a compiler. Publish a new immutable fork release only after
   implementation approval and verification; never replace prior assets.

## Expected files

- `src/script/ast.rs`, `src/script/parser.rs`, script serialization location,
  and `src/lib.rs`: helper contract and existing-event expansion.
- `src/runner.rs`, `src/record.rs`: actual event timing and live recording.
- `src/pointer.rs`: shared visual evaluator; geometry remains centralized.
- `src/recording.rs`, `src/full_recording.rs`, `src/render_json.rs`: complete
  feedback-data propagation and replay.
- `src/render_gif.rs`, `src/render_svg.rs` and relevant renderer glue: shared
  presentation, dirty bounds and frame retention. Use exact edits in large files.
- Existing nearby tests plus a narrowly scoped click fixture when necessary.
- README, FORK, bundled skill, quickstart and release packaging guidance.

Avoid a new file when the existing module is a clear owner. Do not expand the
plan into a framework, broad refactor, or unrelated release automation.

## Completion gate

One normal tape command can author a natural click without hand-timed movement
legs; ordinary Click also receives consistent time-based feedback. Both input
and visuals are evidenced, the operator approves the appearance, and a new
prebuilt release carries matching docs and skill. Until then, this remains a
plan or work in progress—not a claim that the current release implements it.

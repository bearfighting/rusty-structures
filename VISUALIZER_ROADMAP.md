# Data Structure Visualizer — roadmap

The roadmap is organized around reviewable learning outcomes. Each stage
builds on the same typed command and trace boundary.

## Stage 0 — design and interface contract

- Agree the first lesson, operation set, trace event vocabulary, and screen
  regions.
- Keep the Rust core independent from browser and UI dependencies.
- Define stable visual node identity and replay/reset semantics.

**Done when:** the scope and trace contract are recorded in
`VISUALIZER_DESIGN.md` and this roadmap.

## Stage 1 — BST vertical slice

- Add a Wasm adapter that invokes the repository's real BST.
- Record compare, descend, mutation, and completion events from the algorithm.
- Return a serializable visual snapshot and trace.
- Build a Vite + React + TypeScript page with an SVG BST canvas, operation
  controls, step/back/play/pause/reset, and an explanation panel.
- Add reduced-motion behavior and keyboard-operable controls.

**Done when:** a user can insert, search, and remove keys; step through each
recorded algorithm event; and see the final canvas agree with Rust.

## Stage 2 — teaching quality

- Add pseudocode highlighting and live BST ordering checks.
- Add examples for all delete shapes and duplicates/missing keys.
- Improve focus, responsive layout, and explanation language from learner
  feedback.

**Done when:** the key algorithm paths are easy to follow without relying on
animation timing or source-code familiarity.

## Stage 3 — B+ tree lesson

- Add a B+ tree session using the arena-backed implementation.
- Visualize internal separators, leaf entries, `next` links, split, borrow,
  merge, and root contraction.
- Use generation-aware visual IDs and check that leaf links match tree order.

**Done when:** users can replay insertion and deletion while seeing both the
search index and leaf scan chain remain consistent.

## Stage 4 — broaden the lesson set

- Add AVL rotations, then red-black recoloring/rotations.
- Add lesson chapters, practice prompts, and shareable operation sequences.
- Consider a Worker only if trace generation for realistic examples blocks the
  UI; defer canvas rendering until SVG has a measured limit.

**Done when:** every lesson reuses the same controls, timeline, and trace
semantics where those semantics genuinely match.

## First delivery boundary

Stage 1 is the first prototype release. It deliberately excludes user accounts,
cloud storage, collaboration, arbitrary data types, a generic graph editor, and
benchmarking. Those features do not help prove the central lesson loop.

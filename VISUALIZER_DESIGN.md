# Data Structure Visualizer — prototype design

## Goal

Build a browser-based learning tool where an operation on a real data structure
drives the animation, current-step explanation, pseudocode highlight, and
invariant display. The first prototype proves this loop for a binary search
tree (BST); B+ tree is the next showcase for multiway splits and linked leaves.

The tool teaches algorithm behavior and Rust data-structure concepts. It is not
a general-purpose graph editor or a performance benchmark.

## Prototype scope

The first usable slice supports an integer-key BST and these operations:

- Insert a key, including duplicate-key behavior.
- Search for a key and show the comparison path.
- Remove a key, including leaf, one-child, two-child, and missing-key paths.
- Step forward/back, play/pause, reset, and adjust playback speed.
- Select a node to inspect its value, parent/child links, and current role.
- Show a short explanation and pseudocode line for the current algorithm event.
- Show whether ordering and reachable-node-count invariants hold.

Initial data sets stay small (around 20 keys) so each operation remains legible.
Free-form dragging of nodes is out of scope; the layout follows the tree.

## User experience

The page has three regions:

1. **Tree canvas:** SVG nodes, edges, search path, active comparison, and
   insertion/removal transitions.
2. **Controls:** operation selector, key input, run/step/back/pause/reset, speed,
   and a small set of useful example sequences.
3. **Lesson panel:** current event, plain-language explanation, pseudocode, and
   invariant status.

Animation is optional. Every action must remain understandable with motion
reduced or playback paused. Keyboard-operable controls and readable labels are
part of the prototype, not later polish.

## Technical architecture

```text
React + TypeScript UI
    ├── controls and lesson state
    ├── SVG renderer and deterministic tree layout
    └── timeline reducer (step, back, play, pause, reset)
             │ typed commands / typed trace data
             ▼
Rust/Wasm adapter
    ├── runs the repository's actual BST implementation
    ├── records semantic algorithm events
    └── returns a visual snapshot plus the operation trace
             │
             ▼
Existing Rust data-structures crate
```

The web app uses Vite for development/build and React with TypeScript for the
interface. SVG is the initial renderer; a small in-project layout function
positions nodes by depth and in-order rank. Animation uses the browser frame
loop and does not determine algorithm state.

The Wasm adapter is a separate boundary so the core crate does not depend on
React, browser APIs, or serialization. It exposes concrete lesson sessions
(initially integer keys), not arbitrary Rust generics. `wasm-bindgen` provides
the Rust/JavaScript boundary; Serde-based conversion is limited to the adapter.

## Trace contract

One user command produces a finite, deterministic trace. Event kinds describe
algorithm meaning rather than animation frames:

```text
Compare { current_node, key, ordering }
Descend { from, to }
CreateNode { node }
ReplaceWithSuccessor { removed_node, successor_node }
UnlinkNode { node }
Finish { result }
```

Each step carries the visual state needed to display that point in the lesson,
the active node IDs, a short explanation key, and a pseudocode step ID. The UI
replays this trace; it never reimplements BST decisions.

Visual node IDs must not become ambiguous when storage is reused. The adapter
uses stable, generation-aware IDs (or monotonic IDs scoped to one lesson
session), even if a Rust structure recycles an internal arena slot.

The first prototype may store a compact snapshot at each step because examples
are small. If traces grow, switch to reversible deltas with periodic snapshot
checkpoints without changing the event meanings.

## Quality checks

- Rust tests continue to verify BST contents and invariants after operations.
- Trace tests replay the recorded operation and assert the final visual keys
  match the actual Rust tree.
- Frontend reducer tests cover forward/back/reset and invalid commands.
- One browser flow checks insert → step through comparisons → finish → search.
- The invariant panel is computed from the visual snapshot and can be compared
  with the Rust result in adapter tests.

## Initial decisions

- Start with BST to prove the end-to-end interaction loop; implement B+ tree
  visualization after the trace and layout contract settles.
- Keep lessons and event explanations in the frontend; keep mutation decisions
  and structural snapshots rooted in the Rust implementation.
- Avoid a backend service, account system, and persistence in the prototype.
- Keep the existing Rust library usable without enabling browser dependencies.

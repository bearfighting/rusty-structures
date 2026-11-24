# Data Structure Visualizer — product and technical design

## Goal

Build a browser learning tool where operations on the repository's real Rust
data structures drive the displayed state and trace. Learners should be able to
choose a structure and implementation, perform meaningful operations, inspect
the result, and replay the algorithm steps without needing to read Rust source.
This is a teaching tool, not a general graph editor or performance benchmark.

The visualizer will grow by structure families. A shared application shell
provides navigation, input, playback, and lesson layout; each family supplies a
view and operation lessons suited to its shape. Avoid a universal renderer or
event vocabulary that hides meaningful algorithm differences.

## Structure catalog and navigation

The catalog groups lessons by family and distinguishes the abstract structure
from the concrete Rust implementation. For example, “Search trees” contains
BST, AVL, and red-black implementations; “Maps and sets” can expose different
backends implementing the same abstract behavior.

The navigation should:

- Offer category and implementation selection in a persistent sidebar or
  header menu, with a clear current selection and breadcrumbs/title.
- Show which lessons are available and which are planned; planned entries are
  informational and cannot be selected as if implemented.
- Show each implementation's supported operations and view modes before the
  learner starts.
- Keep the selected lesson in the URL when practical, so a lesson can be
  bookmarked and browser navigation works.
- Preserve each lesson's in-memory state while switching between lessons in
  the same visit. Provide explicit per-lesson reset and example actions; do not
  silently discard a structure when the learner changes selection.

The shared shell stays in a consistent layout: structure navigation, operation
controls, visualization, trace timeline, and lesson explanation. Controls and
input fields are rendered from the selected lesson's capabilities rather than
showing one universal set of operations.

## View families

Use a discriminated snapshot/view contract so a renderer can select the right
layout without pretending every structure is a tree:

| View kind | Initial structures | Useful details |
| --- | --- | --- |
| `sequence` | Vector, Stack, Queue, Deque | indices, capacity, logical front/back, active range |
| `linkedList` | SinglyLinkedList, DoublyLinkedList | node identity, next/previous links, head/tail |
| `binaryTree` | BST, AVL, RedBlackTree | parent/child links, balance/color metadata, traversal |
| `multiwayTree` | BTree, BPlusTree | keys per node, child ranges, leaf chain for B+ tree |
| `heap` | BinaryHeap, PriorityQueue | array positions and parent/child view, heap order |
| `hashTable` | HashMap, HashSet | buckets, entries, collisions, resize state |
| `trie` | Trie | character/edge labels, terminal nodes, prefix paths |

Some structures may offer multiple render modes, such as array and tree modes
for a heap. Renderers must have a compact mode or a clear size limit so large
structures remain legible and responsive.

Visual IDs represent logical visual elements, not their values. IDs must stay
stable through a trace and must not collide when values repeat or storage slots
are reused. Use session-scoped monotonic IDs or generation-aware IDs; document
when an element is created or removed.

## Operations, input, and trace contract

Each catalog entry declares its own operation descriptors and input schema.
Input forms should cover the real lessons: integer or string key, key/value,
index, and optional initial sequences. Validate at the UI boundary and return
actionable errors. Numeric bounds must match the Rust/WASM type; for example,
an `i32` command must reject values outside the signed 32-bit range rather than
accepting every JavaScript safe integer.

A command produces a finite deterministic trace from the actual Rust
implementation. The Rust adapter owns mutation and algorithm decisions. It may
emit semantic events around an existing operation or call a traced Rust API;
the UI must not independently reimplement the data structure to decide what
happened. Events describe teaching-relevant steps, not animation frames.

Use a small shared event envelope for playback metadata (event identity,
affected visual IDs, explanation key, pseudocode step ID) while preserving
structure-specific event kinds and payloads. A BST comparison, AVL rotation,
hash collision, and queue dequeue are different concepts and should remain
different events.

Snapshots and traces are discriminated by structure/view kind. For small
lessons, full snapshots per step are easiest to inspect and replay. Start with
before/after snapshots plus semantic events where that is sufficient; add
per-step snapshots or reversible deltas when a lesson needs intermediate
structural states. If trace size becomes large, use periodic checkpoints and
deltas without changing event meaning. Set and document a reasonable item/trace
limit for interactive lessons.

Playback state (cursor, play/pause, speed) is UI state. It must be deterministic
when stepping, scrubbing, pausing, or using reduced motion. Reduced-motion
users can inspect every step without automatic animation.

## Architecture

```text
React application shell
  ├── structure catalog and navigation
  ├── operation form generated from lesson capabilities
  ├── shared trace timeline and playback controls
  ├── view-family renderers
  └── structure-specific lesson copy and invariants
             │ typed adapter/session boundary
             ▼
Rust/WASM visualizer crate
  ├── shared serialization and protocol support
  └── one adapter module per structure or implementation
             │
             ▼
Existing Rust data-structures crate
```

The core data-structures crate remains independent of browser, React, and
serialization dependencies. Trace APIs belong with a Rust implementation when
they express useful algorithm steps; the WASM crate owns input conversion,
view snapshots, JavaScript exports, and serialization. Keep structure-specific
events and adapters in separate modules. Add shared abstractions only after at
least two real adapters demonstrate the same need.

When a visualizer needs internal layout that the public collection API does not
expose, provide an owned, read-only snapshot of node IDs, separators, children,
and links. Keep mutable node storage private; the WASM adapter can validate the
snapshot against normal Rust iteration before converting it to a UI view.
Trie snapshots also copy each character prefix, terminal marker, and ordered
child edge so the visualizer can explain shared prefixes and safe pruning
without exposing mutable trie nodes.

Hash-table lessons use a deterministic standard hasher in their Rust session so
bucket positions and collision examples can be replayed consistently. The UI
should present these positions as properties of that configured hasher, not as
a promise about the default randomized hasher's bucket order.

On the web side, a catalog descriptor connects an ID and label to supported
operations, an adapter/session, a view kind, and lesson metadata. Adapter,
renderer, and lesson types should support structure-specific snapshot and trace
types; avoid a single `key: number` and `TreeView` contract as the permanent
boundary. The initial BST implementation may use the narrower types while the
next lesson validates how they generalize.

## Accessibility, responsiveness, and errors

- All actions are keyboard operable, visibly focusable, and have descriptive
  accessible names.
- Provide reduced-motion behavior; meaning must not depend on color or motion.
- Show operation errors next to the relevant control and preserve the current
  structure state when an input is rejected.
- Layouts work on narrow screens and support large but bounded examples.
- Explanations use plain language and remain useful with playback paused.

## Correctness and quality gates

For each lesson, verify that:

- Rust operation tests still establish contents and invariants.
- Adapter/trace checks compare the visual final state with the actual Rust
  structure and validate IDs, event order, and reported size.
- Frontend checks cover input validation, catalog selection, state preservation,
  playback/scrubbing/reset, and view-specific rendering.
- At least one browser flow performs a representative operation and replays
  steps from start to finish.
- Accessibility checks cover keyboard navigation, labels, reduced motion, and
  responsive layout.

## Scope decisions

- Deliver in vertical slices by structure family, starting from simple linear
  structures after the current BST baseline.
- Ship only operations whose Rust behavior, trace, rendering, and lesson text
  are complete; do not advertise an unfinished implementation as available.
- Defer accounts, cloud persistence, collaboration, arbitrary user code,
  benchmarking, and a general graph editor. Revisit only when they support a
  specific learning goal.

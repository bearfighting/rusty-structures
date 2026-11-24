# Data Structure Visualizer — roadmap

The roadmap grows the visualizer by structure family. Each stage is a complete
vertical slice: Rust behavior, WASM adapter, typed view/trace, UI, lesson
content, and correctness checks. Finish one representative structure in a
family before adding its close variants. Do not build a universal framework in
advance of a second real use case.

## Stage 0 — product and protocol foundation

- Record the structure catalog, navigation behavior, view families, typed
  inputs, trace envelope, stable visual identity, and quality gates in
  `VISUALIZER_DESIGN.md`.
- Keep the Rust crate independent of browser/UI dependencies. Keep one
  structure-specific module per WASM adapter and a small shared serialization
  boundary.
- Define a catalog descriptor for lesson identity, category, implementation,
  supported operations, input schema, view kind, and renderer/lesson binding.
- Replace the permanent single-key/single-tree assumption with discriminated
  view and command types as the next structure requires them.
- Add structure navigation to the shared shell. Show available versus planned
  lessons clearly, and preserve each lesson's in-memory state on navigation.

**Done when:** a learner can navigate the catalog, the existing BST is loaded
through its adapter, state does not disappear on lesson switches, and the
protocol can represent at least one non-tree view without weakening BST types.

**Current progress:** the catalog, available/planned states, hash-based lesson
route, per-lesson WASM session cache, typed command inputs, and `sequence` view
shape are in place. BST is the only runnable lesson today, so cross-lesson
state restoration still needs a hands-on check when the next lesson becomes
available.

## Stage 1 — BST baseline and adapter boundary

- Keep the current insert, search, and remove lesson as the reference vertical
  slice.
- Retain the new `BstSession` adapter module and shared serialization module;
  do not mix later structures into the BST event types.
- Make visual identity explicit and session-stable instead of deriving node IDs
  from values.
- Align accepted numeric inputs with Rust `i32` bounds and show useful errors.
- Verify playback, scrubbing, resetting, keyboard controls, reduced motion,
  and responsive behavior.
- Check visual state and event claims against the Rust result.

**Done when:** BST behavior remains correct through the catalog and all stage
quality gates pass. This is the baseline for later family adapters.

**Current progress:** the WASM adapter assigns monotonic session-scoped node
IDs and includes them in snapshots and node-related events; the UI validates
the signed `i32` input range. Production build and Rust formatting pass.
Browser-level replay, keyboard, reduced-motion, and responsive checks remain
part of the stage acceptance before calling this stage complete.

## Stage 2 — linear containers

Representative first slice: `Vector`, followed by `Stack`, `Queue`, and
`Deque`.

- Build a reusable sequence renderer with index/position labels, active ranges,
  and capacity where applicable.
- Add descriptors and input forms for value, index, and optional batches.
- Implement meaningful operation traces: vector insert/remove/access/reserve;
  stack push/pop/peek; queue enqueue/dequeue/front; deque operations at both
  ends. Expose only operations supported by the actual Rust API.
- Explain bounds/capacity behavior and error cases, including empty removal and
  bounded-capacity failures where relevant.

**Done when:** Vector proves the sequence view and indexed operations; the
remaining linear containers reuse it without awkwardly pretending to be trees
or exposing unsupported operations.

**Current progress:** Vector is available with push, pop, indexed insert,
indexed remove, indexed read, and reserve. Stack, Queue, and Deque now reuse
the sequence view with endpoint markers and only expose operations supported
by each Rust API. Stack shows its top and LIFO operations; Queue shows front to
back FIFO behavior; Deque exposes both ends. All four sessions preserve stable
item IDs and report capacity changes where the underlying API exposes capacity.
Browser-level replay and accessibility checks remain part of stage acceptance.

## Stage 3 — linked lists

Representative slices: `SinglyLinkedList` and `DoublyLinkedList`.

- Reuse sequence concepts but add explicit node IDs and next/previous links.
- Show head/tail, traversal direction, insertion/removal at meaningful
  positions, and the nodes/links changed by an operation.
- Validate reachable-node count and link consistency in adapter checks.

**Done when:** each list lesson makes its pointer/link changes understandable
and distinguishes singly from doubly linked behavior.

**Current progress:** `SinglyLinkedList` and `DoublyLinkedList` are available
with push-front, push-back, pop-front, pop-back (doubly linked only), front,
and back lessons. The shared view shows stable node IDs, HEAD/TAIL, and the
Rust-reported pointer transitions; the doubly linked view also renders and
traces reverse `prev` links. Browser-level flow and accessibility checks remain
part of stage acceptance.

## Stage 4 — heaps and priority queues

Representative first slice: `BinaryHeap`, followed by `PriorityQueue`.

- Add a heap view with synchronized array and tree modes.
- Trace comparisons, swaps, sift-up/sift-down, and root removal from the Rust
  operation path.
- Show min/max behavior and explain the priority ordering used by the wrapper.

**Done when:** array positions and tree relationships stay synchronized through
  insert, peek, and removal traces; the heap invariant is checked.

**Current progress:** BinaryHeap and PriorityQueue are available with their
respective push/pop/peek APIs. Both lessons synchronize the Rust heap array and
tree layout, and trace insertion, comparisons, swaps, and root removal. The
priority queue view shows both priority and payload, states that equal-priority
ordering is unspecified, and validates the adapter's heap layout against
`PriorityQueue::iter()`. Browser-level acceptance checks remain for this stage.

## Stage 5 — search-tree variants

Add `AvlTree`, then `RedBlackTree`, building on the BST binary-tree view.

- Include balance factor/height for AVL and color/black-height concepts for
  red-black trees.
- Show rotations and recoloring as explicit events and intermediate visual
  states.
- Keep operation semantics shared where appropriate, while keeping each
  balancing algorithm's events and invariant checks specific.

**Done when:** users can compare the same operation across BST, AVL, and
red-black implementations and see their distinct balancing work.

**Current progress:** AVL and left-leaning `RedBlackTree` are available for
insertion, search, and removal. AVL displays node heights/balance factors and
traces its four rotation cases. Red-black displays node colors and traces
rotations, color flips, red-link moves, and successor selection. Both adapters
check their simulated structure against Rust results and their own balance
invariants. Browser-level checks remain before marking the stage accepted.

## Stage 6 — multiway trees

Add `BTreeMap` / `BTreeSet`, then `BPlusTreeMap` / `BPlusTreeSet`.

- Add a multiway-node renderer for grouped keys and child ranges.
- For B+ trees, also show linked leaves and support range/ordered scans where
  the Rust API allows them.
- Trace splits, merges, borrowing, root changes, and leaf-link changes.
- Bound example sizes and use compact labels to keep nodes legible.

**Done when:** tree structure and B+ leaf-chain invariants are checked against
the Rust implementation through representative insertions and deletions.

**Current progress:** `BTreeMap<i32, i32>`, `BTreeSet<i32>`,
`BPlusTreeMap<i32, i32>`, and `BPlusTreeSet<i32>` are available with
structure-appropriate insert, lookup, and removal operations. B-tree lessons
show multi-key nodes; B+ lessons distinguish internal indexes from data leaves,
render the linked leaf chain, and support ordered scans. The WASM adapters use
the Rust implementations and check snapshots against ordered Rust iteration,
occupancy, child counts, separator ranges, equal leaf depth, and leaf links.
Automated production builds and Rust checks pass; browser-level replay and
accessibility checks remain part of stage acceptance.

## Stage 7 — hash structures and maps

Add `HashMap`, then `HashSet`; add tree map wrappers when their backend lessons
are available.

- Build a bucket/table view showing entries, collisions, and resize/rehash
  events when the implementation exposes those steps.
- Use typed key/value inputs and explain equality/hash constraints in learner
  language.
- Present `HashSet` as a set-focused lesson and distinguish abstract map/set
  behavior from the selected backend implementation.

**Done when:** lookups and mutations can be followed through buckets, and
reported length and collision/resize state match Rust.

**Current progress:** `HashMap<i32, i32>` and `HashSet<i32>` are available with
insert, lookup, and removal lessons. The bucket view shows collisions, load
capacity, growth, and rehashed keys; the adapters use a reproducible hasher and
check positions against the Rust collections. `BinarySearchTreeMap`,
`AvlTreeMap`, and `RedBlackTreeMap` are also available as ordered map lessons;
their shared map view shows key-sorted entries and the backend implementation.
`RecursiveBinarySearchTreeMap` remains planned until its backend lesson is
available. Browser-level replay and accessibility checks remain part of stage
acceptance.

## Stage 8 — Trie and remaining implementations

- Add a prefix-tree renderer with edge labels, terminal markers, and shared
  prefix highlighting.
- Trace insert, prefix lookup, exact lookup, and removal/pruning where exposed.
- Review the full public Rust API inventory and record any intentionally
  excluded structure or operation with a reason.

**Done when:** Trie lessons explain prefix behavior, and every public structure
is either represented in the catalog or explicitly deferred with rationale.

**Current progress:** Trie is available with insert, exact lookup, prefix
lookup, prefix result scanning, and removal. Its renderer labels Unicode
character edges, marks complete words, highlights the active path, and traces
terminal removal and pruned nodes. The adapter checks the snapshot's terminal
words and length against Rust iteration. The public API inventory is represented
in the catalog: Stack, Queue, Deque, and PriorityQueue are now available.
Recursive BST variants remain planned; recursive map variants wait on their
recursive tree backend. `TreeMap` is a shared interface rather than a separate
storage structure, so its concrete implementations are the lessons.
Browser-level flow, accessibility, and responsive checks remain before stage
acceptance.

## Ongoing quality gates

Apply these to every stage:

- Rust operation and invariant checks remain authoritative.
- Adapter checks verify trace order, stable IDs, final state, sizes, and
  structure-specific invariants.
- UI checks cover invalid input, operation availability, state preservation,
  playback controls, and view rendering.
- At least one browser flow exercises the new family's representative lesson.
- Keyboard use, accessible labels, reduced motion, narrow screens, and bounded
  large examples are reviewed before marking a lesson available.

## Deferred until justified

Do not start with a worker, canvas renderer, persisted user accounts, cloud
sharing, arbitrary user-defined structures, or benchmarking. First measure a
real limit or identify a concrete learning outcome that needs one of these.

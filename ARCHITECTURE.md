# Codebase architecture

The source tree groups structures by domain. Each domain module owns its
public API, its implementation, and any contracts that describe behavior
shared by multiple implementations. The crate facade preserves the established
root module paths while also exposing the `linear` and `range` groupings.

## Source layout

```text
src/
  lib.rs                    crate facade and compatibility re-exports
  algebra/                  reusable algebraic contracts (Monoid, TryMonoid)
  bit_set/                  packed set of non-negative integer indices
  bloom_filter/             probabilistic set-membership filter
  lru_cache/                hash-indexed cache with recency ordering
  foundation/               crate-private storage and shared errors
    ring_buffer.rs          storage used by Queue and Deque
  linear/                   sequential containers
    mod.rs
    vector.rs               concrete growable-array implementation
    vector/                 vector storage, iterators, errors, standard traits
    stack.rs                Vector-backed LIFO adapter
    queue.rs                RingBuffer-backed FIFO adapter
    fixed_capacity_queue.rs fixed-capacity FIFO adapter
    deque.rs                RingBuffer-backed double-ended adapter
    queue/ deque/ stack/    focused iterators, errors, and trait implementations
  heap/                     heap and priority-queue family
    mod.rs                  family exports
    binary_heap.rs          binary max-heap
    dary_heap.rs            configurable d-ary max-heap
    index_priority_queue.rs indexed priority updates
    priority_queue.rs       item/priority adapter
  hash/                     map, set adapter, iterators, and traits
  skip_list/                probabilistic ordered map and set
  tree/                     search, balanced, multiway, prefix, and map types
  graph/                    graph representations, view traits, and algorithms
  range/                    disjoint-set and range-query structures
```

The large domain modules use `mod.rs` as their entry point, with implementation
files and focused child modules beside it. `linear/` and `range/` additionally
group related top-level structures without changing their established public
paths. For example, both `data_structures::vector::Vector` and
`data_structures::linear::vector::Vector` resolve to the same type.

## Relationships and dependency direction

```text
crate facade (`lib.rs`)
      │ re-exports stable public modules and types
      ▼
domain APIs
  ├── linear: Vector, Stack, Queue, Deque
  ├── heap: BinaryHeap, DaryHeap, PriorityQueue, IndexPriorityQueue
  ├── hash: HashMap, HashSet
  ├── tree: search trees/maps/sets, Trie, RadixTrie, persistent Trie and AVL map
  ├── graph: representations + representation-independent algorithms
  └── range: DisjointSet, FenwickTree, SegmentTree, LazySegmentTree
      │
      ├── adapters compose concrete structures
      │     Stack → Vector; Queue/Deque → RingBuffer
      │     PriorityQueue → BinaryHeap; HashSet → HashMap
      │     tree sets/maps → their tree backends
      │
      └── shared internals
            foundation/ → crate-private allocation errors and RingBuffer
```

Dependencies point from adapters and algorithms toward the concrete structures
they use. A backing structure does not depend on an adapter. Shared code belongs
in `foundation/` only when multiple domains reuse it; one-off helpers stay next
to their implementation.

## Interfaces and contracts

Contracts live beside the domain they describe, rather than in a global trait
layer:

- `graph/traits.rs` defines `GraphView` and `WeightedGraphView`; graph algorithms
  use these traits instead of depending on one storage representation.
- `tree/map.rs` defines `TreeMap`, the common ordered key-value API implemented
  by the tree map variants.
- `algebra/` defines `Monoid` for total associative operations and `TryMonoid`
  for aggregate operations that can report errors.
- `range/segment_tree.rs` consumes `TryMonoid`; total `Monoid` implementations
  receive an infallible `TryMonoid` implementation automatically.
- `range/lazy_segment_tree.rs` defines `LazySegmentTreeSpec`, which combines
  aggregate and deferred-update behavior for the advanced tree.
- Per-structure `traits.rs` files implement standard Rust traits and iterators;
  they are not cross-domain abstraction layers.

Introduce a shared contract only when implementations have the same semantics.
Matching method names alone do not justify a common trait.

## Public API and implementation files

- `src/lib.rs` documents the crate, exposes domain modules, and re-exports
  commonly used types. Domain paths (for example, `data_structures::range::FenwickTree`)
  are canonical; root type exports are conveniences. Compatibility module
  re-exports preserve paths such as `data_structures::queue::Queue`.
- Algebraic contracts are owned by `algebra`; range structures consume them.
  Import `Monoid` and `TryMonoid` from `algebra` (or the crate root), rather
  than through a range-query implementation module.
- A domain's `mod.rs` selects its public child modules and exports. Concrete
  types, node layouts, storage arrays, and mutation helpers remain private
  unless a documented public API requires them.
- Iterators, errors, and standard trait implementations get separate files
  when doing so makes the primary implementation easier to follow.
- `Vector<T>` is both a public collection and a backing structure. `RingBuffer`
  is crate-private because it exists to support the queue adapters.

## Adding a structure

1. Put it in the domain that describes its behavior; add a domain folder when
   multiple related structures benefit from a visible grouping.
2. Keep its public contract and implementation together. Put a shared trait in
   that domain only when multiple implementations need the same semantics.
3. Compose an existing structure when it already supplies the needed storage
   or invariant; keep the adapter near its public type.
4. Update the domain's `mod.rs`, crate facade when appropriate, and this map if
   the source layout changes.

The project remains a single library crate. The directory structure makes
teaching relationships visible without adding crate boundaries or a general
framework that current implementations do not need.

# Codebase architecture

This crate is organized by data-structure domain, with a small number of
explicit layers inside those domains. The layers clarify ownership and reuse;
they are not separate crates or a general-purpose framework.

## Layers

### Public API

- `src/lib.rs` is the crate facade. It documents the project and re-exports the
  stable, commonly used types.
- Domain modules (`vector`, `list`, `hash`, `heap`, `tree`, and so on) expose
  structure-specific APIs and Rustdoc.
- Shared behavior contracts belong beside the relevant domain. For example,
  `tree::map::TreeMap` describes the common operations supported by ordered
  tree maps.

### Adapters and composed structures

These types provide a useful semantic interface by composing another
structure, rather than duplicating its storage algorithm:

- `Stack<T>` stores values in `Vector<T>`.
- `Queue<T>` and `Deque<T>` use the shared ring buffer.
- `PriorityQueue<T>` uses `BinaryHeap<T>`.
- `HashSet<T>` uses `HashMap<T, ()>`.
- `BTreeSet<T>` uses `BTreeMap<T, ()>`.
- `BPlusTreeSet<T>` uses `BPlusTreeMap<T, ()>`.
- The `*TreeMap` variants adapt the set-like BST, AVL, and red-black tree
  implementations to key-value behavior. `BTreeMap` and `BPlusTreeMap`
  implement `TreeMap` directly because they already store key-value entries.

Adapter code stays close to its public domain module so readers can find an
abstraction and its implementation together.

### Foundation and concrete implementations

- `src/foundation/` contains shared internals used by multiple public
  structures: allocation errors and the ring buffer. These are crate-private.
- `src/vector.rs`, `src/hash/map.rs`, and `src/tree/*.rs` contain concrete
  storage and algorithms. Node layouts and mutation helpers remain private;
  child modules hold iterators or focused implementation details where useful.
- `Vector<T>` is both a public foundational collection and the storage used by
  several other structures. Its allocation details live under `vector/`.

## Dependency direction

```text
public domain APIs
        ↓
composed adapters (Stack, Queue, sets, priority queue, tree maps)
        ↓
concrete structures and shared foundation
```

Dependencies should point downward. A foundational structure does not depend
on an adapter that uses it. Shared code moves into `foundation` only when
multiple structures genuinely reuse it; one-off helpers stay with their
implementation.

## Adding a structure

1. Put its public type and API in the matching domain module.
2. Keep representation details private and split child files only when they
   clarify a real boundary, such as iterators or errors.
3. Compose an existing structure when it already provides the needed storage
   or invariant. Keep the adapter in its domain module.
4. Add a shared trait only when implementations have the same semantics, not
   merely similarly named methods.
5. Preserve the crate facade and stable module paths when moving internals.

The project remains a single library crate. This layout is intended to make
teaching relationships visible without introducing extra crates or abstraction
layers that do not serve a current implementation.

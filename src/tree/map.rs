//! A shared ordered-map interface backed by different tree implementations.

use super::{AvlTree, BTreeMap, BinarySearchTree, RecursiveBinarySearchTree, RedBlackTree};
use std::cmp::Ordering;
use std::iter::FusedIterator;
use std::marker::PhantomData;

/// Common ordered key-value operations implemented by the tree map variants.
pub trait TreeMap<K: Ord, V> {
    /// An in-order iterator over key-value pairs.
    type Iter<'a>: Iterator<Item = (&'a K, &'a V)> + ExactSizeIterator + FusedIterator
    where
        Self: 'a,
        K: 'a,
        V: 'a;

    /// Returns the number of entries.
    fn len(&self) -> usize;
    /// Returns whether the map is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Inserts a pair and returns the previous value for the key, if any.
    fn insert(&mut self, key: K, value: V) -> Option<V>;
    /// Returns the value associated with `key`.
    fn get(&self, key: &K) -> Option<&V>;
    /// Returns mutable access to the value associated with `key`.
    fn get_mut(&mut self, key: &K) -> Option<&mut V>;
    /// Returns whether `key` is present.
    fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }
    /// Removes a pair and returns its value.
    fn remove(&mut self, key: &K) -> Option<V>;
    /// Iterates over pairs in ascending key order.
    fn iter(&self) -> Self::Iter<'_>;
}

#[derive(Debug)]
struct Entry<K, V> {
    key: K,
    value: V,
}
impl<K: PartialEq, V> PartialEq for Entry<K, V> {
    fn eq(&self, rhs: &Self) -> bool {
        self.key == rhs.key
    }
}
impl<K: Eq, V> Eq for Entry<K, V> {}
impl<K: PartialOrd, V> PartialOrd for Entry<K, V> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        self.key.partial_cmp(&rhs.key)
    }
}
impl<K: Ord, V> Ord for Entry<K, V> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.key.cmp(&rhs.key)
    }
}

trait PairIterator<'a, K: 'a, V: 'a>:
    Iterator<Item = (&'a K, &'a V)> + ExactSizeIterator + FusedIterator
{
}
struct RawMapIter<'a, K, V, I> {
    inner: I,
    marker: PhantomData<(&'a K, &'a V)>,
}
impl<'a, K, V, I> RawMapIter<'a, K, V, I> {
    fn new(inner: I) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }
}
impl<'a, K, V, I: Iterator<Item = &'a Entry<K, V>>> Iterator for RawMapIter<'a, K, V, I> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|e| (&e.key, &e.value))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
impl<'a, K, V, I: ExactSizeIterator<Item = &'a Entry<K, V>>> ExactSizeIterator
    for RawMapIter<'a, K, V, I>
{
}
impl<'a, K, V, I: FusedIterator<Item = &'a Entry<K, V>>> FusedIterator for RawMapIter<'a, K, V, I> {}
impl<'a, K, V, I> PairIterator<'a, K, V> for RawMapIter<'a, K, V, I> where
    I: Iterator<Item = &'a Entry<K, V>> + ExactSizeIterator + FusedIterator
{
}

/// In-order iterator used by the set-backed map adapters.
pub struct MapIter<'a, K, V> {
    inner: Box<dyn PairIterator<'a, K, V> + 'a>,
}
impl<'a, K, V> MapIter<'a, K, V> {
    fn new<I>(inner: I) -> Self
    where
        I: Iterator<Item = &'a Entry<K, V>> + ExactSizeIterator + FusedIterator + 'a,
    {
        Self {
            inner: Box::new(RawMapIter::new(inner)),
        }
    }
}
impl<'a, K, V> Iterator for MapIter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
impl<K, V> ExactSizeIterator for MapIter<'_, K, V> {}
impl<K, V> FusedIterator for MapIter<'_, K, V> {}

trait TreeBackend<T: Ord> {
    fn new() -> Self;
    fn insert(&mut self, value: T) -> bool;
    fn get_by<Q: ?Sized>(&self, query: &Q, cmp: impl Fn(&Q, &T) -> Ordering) -> Option<&T>;
    fn get_mut_by<Q: ?Sized>(
        &mut self,
        query: &Q,
        cmp: impl Fn(&Q, &T) -> Ordering,
    ) -> Option<&mut T>;
    fn remove_by<Q: ?Sized>(&mut self, query: &Q, cmp: impl Fn(&Q, &T) -> Ordering) -> Option<T>;
}

// The backend operations are supplied by each tree module, where its private
// node type is visible. This keeps balancing logic in its original algorithm.
impl<T: Ord> TreeBackend<T> for BinarySearchTree<T> {
    fn new() -> Self {
        Self::new()
    }
    fn insert(&mut self, v: T) -> bool {
        self.insert(v)
    }
    fn get_by<Q: ?Sized>(&self, q: &Q, cmp: impl Fn(&Q, &T) -> Ordering) -> Option<&T> {
        self.get_by(q, cmp)
    }
    fn get_mut_by<Q: ?Sized>(&mut self, q: &Q, cmp: impl Fn(&Q, &T) -> Ordering) -> Option<&mut T> {
        self.get_mut_by(q, cmp)
    }
    fn remove_by<Q: ?Sized>(&mut self, q: &Q, cmp: impl Fn(&Q, &T) -> Ordering) -> Option<T> {
        self.remove_by(q, cmp)
    }
}
impl<T: Ord> TreeBackend<T> for RecursiveBinarySearchTree<T> {
    fn new() -> Self {
        Self::new()
    }
    fn insert(&mut self, v: T) -> bool {
        self.insert(v)
    }
    fn get_by<Q: ?Sized>(&self, q: &Q, cmp: impl Fn(&Q, &T) -> Ordering) -> Option<&T> {
        self.get_by(q, cmp)
    }
    fn get_mut_by<Q: ?Sized>(&mut self, q: &Q, cmp: impl Fn(&Q, &T) -> Ordering) -> Option<&mut T> {
        self.get_mut_by(q, cmp)
    }
    fn remove_by<Q: ?Sized>(&mut self, q: &Q, cmp: impl Fn(&Q, &T) -> Ordering) -> Option<T> {
        self.remove_by(q, cmp)
    }
}
impl<T: Ord> TreeBackend<T> for AvlTree<T> {
    fn new() -> Self {
        Self::new()
    }
    fn insert(&mut self, v: T) -> bool {
        self.insert(v)
    }
    fn get_by<Q: ?Sized>(&self, q: &Q, cmp: impl Fn(&Q, &T) -> Ordering) -> Option<&T> {
        self.get_by(q, cmp)
    }
    fn get_mut_by<Q: ?Sized>(&mut self, q: &Q, cmp: impl Fn(&Q, &T) -> Ordering) -> Option<&mut T> {
        self.get_mut_by(q, cmp)
    }
    fn remove_by<Q: ?Sized>(&mut self, q: &Q, cmp: impl Fn(&Q, &T) -> Ordering) -> Option<T> {
        self.remove_by(q, cmp)
    }
}
impl<T: Ord> TreeBackend<T> for RedBlackTree<T> {
    fn new() -> Self {
        Self::new()
    }
    fn insert(&mut self, v: T) -> bool {
        self.insert(v)
    }
    fn get_by<Q: ?Sized>(&self, q: &Q, cmp: impl Fn(&Q, &T) -> Ordering) -> Option<&T> {
        self.get_by(q, cmp)
    }
    fn get_mut_by<Q: ?Sized>(&mut self, q: &Q, cmp: impl Fn(&Q, &T) -> Ordering) -> Option<&mut T> {
        self.get_mut_by(q, cmp)
    }
    fn remove_by<Q: ?Sized>(&mut self, q: &Q, cmp: impl Fn(&Q, &T) -> Ordering) -> Option<T> {
        self.remove_by(q, cmp)
    }
}

#[derive(Debug)]
struct Adapter<K: Ord, V, T: TreeBackend<Entry<K, V>>> {
    tree: T,
    marker: PhantomData<(K, V)>,
}
impl<K: Ord, V, T: TreeBackend<Entry<K, V>>> Adapter<K, V, T> {
    fn new() -> Self {
        Self {
            tree: T::new(),
            marker: PhantomData,
        }
    }
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        if let Some(entry) = self.tree.get_mut_by(&key, |k, e| k.cmp(&e.key)) {
            return Some(std::mem::replace(&mut entry.value, value));
        }
        self.tree.insert(Entry { key, value });
        None
    }
    fn get(&self, key: &K) -> Option<&V> {
        self.tree
            .get_by(key, |k, e| k.cmp(&e.key))
            .map(|e| &e.value)
    }
    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.tree
            .get_mut_by(key, |k, e| k.cmp(&e.key))
            .map(|e| &mut e.value)
    }
    fn remove(&mut self, key: &K) -> Option<V> {
        self.tree
            .remove_by(key, |k, e| k.cmp(&e.key))
            .map(|e| e.value)
    }
}

macro_rules! define_tree_map {
    ($name:ident, $tree:ident) => {
        #[doc = concat!("A key-value map backed by [`", stringify!($tree), "`].")]
        #[derive(Debug)]
        pub struct $name<K: Ord, V> {
            inner: Adapter<K, V, $tree<Entry<K, V>>>,
        }
        impl<K: Ord, V> $name<K, V> {
            pub fn new() -> Self {
                Self {
                    inner: Adapter::new(),
                }
            }
            pub fn len(&self) -> usize {
                self.inner.tree.len()
            }
            pub fn is_empty(&self) -> bool {
                self.len() == 0
            }
            pub fn insert(&mut self, key: K, value: V) -> Option<V> {
                self.inner.insert(key, value)
            }
            pub fn get(&self, key: &K) -> Option<&V> {
                self.inner.get(key)
            }
            pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
                self.inner.get_mut(key)
            }
            pub fn contains_key(&self, key: &K) -> bool {
                self.get(key).is_some()
            }
            pub fn remove(&mut self, key: &K) -> Option<V> {
                self.inner.remove(key)
            }
            pub fn iter(&self) -> MapIter<'_, K, V> {
                MapIter::new(self.inner.tree.iter())
            }
        }
        impl<K: Ord, V> Default for $name<K, V> {
            fn default() -> Self {
                Self::new()
            }
        }
        impl<K: Ord, V> TreeMap<K, V> for $name<K, V> {
            type Iter<'a>
                = MapIter<'a, K, V>
            where
                Self: 'a,
                K: 'a,
                V: 'a;
            fn len(&self) -> usize {
                self.inner.tree.len()
            }
            fn insert(&mut self, k: K, v: V) -> Option<V> {
                self.inner.insert(k, v)
            }
            fn get(&self, k: &K) -> Option<&V> {
                self.inner.get(k)
            }
            fn get_mut(&mut self, k: &K) -> Option<&mut V> {
                self.inner.get_mut(k)
            }
            fn remove(&mut self, k: &K) -> Option<V> {
                self.inner.remove(k)
            }
            fn iter(&self) -> Self::Iter<'_> {
                MapIter::new(self.inner.tree.iter())
            }
        }
    };
}

define_tree_map!(BinarySearchTreeMap, BinarySearchTree);
define_tree_map!(RecursiveBinarySearchTreeMap, RecursiveBinarySearchTree);
define_tree_map!(AvlTreeMap, AvlTree);
define_tree_map!(RedBlackTreeMap, RedBlackTree);

impl<K: Ord, V> TreeMap<K, V> for BTreeMap<K, V> {
    type Iter<'a>
        = super::b_tree::Iter<'a, K, V>
    where
        Self: 'a,
        K: 'a,
        V: 'a;
    fn len(&self) -> usize {
        self.len()
    }
    fn insert(&mut self, k: K, v: V) -> Option<V> {
        BTreeMap::insert(self, k, v)
    }
    fn get(&self, k: &K) -> Option<&V> {
        BTreeMap::get(self, k)
    }
    fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        BTreeMap::get_mut(self, k)
    }
    fn remove(&mut self, k: &K) -> Option<V> {
        BTreeMap::remove(self, k)
    }
    fn iter(&self) -> Self::Iter<'_> {
        BTreeMap::iter(self)
    }
}

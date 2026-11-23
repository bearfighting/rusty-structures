use data_structures::{
    AvlTree, AvlTreeMap, BPlusTreeMap, BPlusTreeSet, BTreeMap, BTreeSet, BinaryHeap,
    BinarySearchTree, BinarySearchTreeMap, Deque, DoublyLinkedList, HashMap, HashSet,
    PriorityQueue, Queue, RecursiveBinarySearchTree, RecursiveBinarySearchTreeMap, RedBlackTree,
    RedBlackTreeMap, SinglyLinkedList, Stack, TreeMap, Trie, Vector,
};
use std::hash::{BuildHasher, Hasher};

#[test]
fn vector_grows_and_keeps_elements_initialized() {
    let mut values = Vector::new();
    values.push(1);
    values.push(3);
    values.insert(1, 2);

    assert_eq!(values.as_slice(), &[1, 2, 3]);
    assert_eq!(values.remove(1), Some(2));
    assert_eq!(values.swap_remove(0), Some(1));
    assert_eq!(values.pop(), Some(3));
    assert_eq!(values.pop(), None);
    assert!(values.is_empty());
}

#[test]
fn vector_drops_each_live_element_once() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    struct CountDrop(Arc<AtomicUsize>);
    impl Drop for CountDrop {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    let drops = Arc::new(AtomicUsize::new(0));
    let mut values = Vector::new();
    for _ in 0..4 {
        values.push(CountDrop(Arc::clone(&drops)));
    }
    drop(values.remove(1));
    values.truncate(1);
    assert_eq!(drops.load(Ordering::SeqCst), 3);
    drop(values);
    assert_eq!(drops.load(Ordering::SeqCst), 4);
}

#[test]
fn queue_wraps_then_grows_without_changing_fifo_order() {
    let mut queue = Queue::with_capacity(3);
    queue.enqueue(1);
    queue.enqueue(2);
    queue.enqueue(3);
    assert_eq!(queue.dequeue(), Some(1));
    queue.enqueue(4); // wraps to the first slot
    queue.enqueue(5); // grows while wrapped

    assert_eq!(queue.iter().copied().collect::<Vec<_>>(), [2, 3, 4, 5]);
    assert_eq!(
        (
            queue.dequeue(),
            queue.dequeue(),
            queue.dequeue(),
            queue.dequeue()
        ),
        (Some(2), Some(3), Some(4), Some(5))
    );
    assert_eq!(queue.dequeue(), None);
}

#[test]
fn deque_mixes_both_ends_across_growth() {
    let mut deque = Deque::with_capacity(2);
    deque.push_back(2);
    deque.push_front(1);
    deque.push_back(3);
    deque.push_front(0);

    assert_eq!(deque.iter().copied().collect::<Vec<_>>(), [0, 1, 2, 3]);
    assert_eq!(deque.pop_back(), Some(3));
    assert_eq!(deque.pop_front(), Some(0));
    assert_eq!(deque.pop_back(), Some(2));
    assert_eq!(deque.pop_front(), Some(1));
    assert!(deque.is_empty());
}

#[test]
fn stack_is_lifo_and_iterates_from_the_top() {
    let mut stack = Stack::new();
    assert_eq!(stack.pop(), None);
    stack.push(1);
    stack.push(2);

    assert_eq!(stack.iter().copied().collect::<Vec<_>>(), [2, 1]);
    assert_eq!(
        (stack.pop(), stack.pop(), stack.pop()),
        (Some(2), Some(1), None)
    );
}

#[test]
fn singly_linked_list_repairs_tail_after_becoming_empty() {
    let mut list = SinglyLinkedList::new();
    list.push_back(1);
    assert_eq!(list.pop_front(), Some(1));
    list.push_back(2);
    list.push_front(0);

    assert_eq!(list.front(), Some(&0));
    assert_eq!(list.back(), Some(&2));
    assert_eq!(list.iter().copied().collect::<Vec<_>>(), [0, 2]);
}

#[test]
fn doubly_linked_list_keeps_both_directions_consistent() {
    let mut list = DoublyLinkedList::new();
    list.push_back(2);
    list.push_front(1);
    list.push_back(3);

    assert_eq!(list.iter().copied().collect::<Vec<_>>(), [1, 2, 3]);
    assert_eq!(list.iter().rev().copied().collect::<Vec<_>>(), [3, 2, 1]);
    assert_eq!(list.pop_back(), Some(3));
    assert_eq!(list.pop_front(), Some(1));
    assert_eq!(list.pop_back(), Some(2));
    assert!(list.is_empty());
}

#[derive(Clone)]
struct ConstantState;

struct ConstantHasher;

impl BuildHasher for ConstantState {
    type Hasher = ConstantHasher;

    fn build_hasher(&self) -> Self::Hasher {
        ConstantHasher
    }
}

impl Hasher for ConstantHasher {
    fn finish(&self) -> u64 {
        0
    }

    fn write(&mut self, _bytes: &[u8]) {}
}

#[test]
fn hash_map_handles_collisions_removal_and_growth() {
    let mut map = HashMap::with_hasher(ConstantState);
    for key in 0..40 {
        assert_eq!(map.insert(key, key * 10), None);
    }
    assert_eq!(map.insert(12, 999), Some(120));
    assert_eq!(map.get(&12), Some(&999));
    assert_eq!(map.remove(&13), Some(130));
    assert_eq!(map.get(&13), None);
    for key in 0..40 {
        if key != 13 {
            assert_eq!(
                map.get(&key),
                Some(&(if key == 12 { 999 } else { key * 10 }))
            );
        }
    }
}

#[test]
fn hash_set_rejects_duplicates_and_removes_values() {
    let mut set = HashSet::with_hasher(ConstantState);
    assert!(set.insert(7));
    assert!(!set.insert(7));
    assert!(set.insert(8));
    assert!(set.remove(&7));
    assert!(!set.contains(&7));
    assert!(set.contains(&8));
}

#[test]
fn binary_heap_preserves_max_heap_order_on_push_and_pop() {
    let mut heap = BinaryHeap::new();
    assert_eq!(heap.pop(), None);
    for value in [4, 9, 1, 7, 9, 2] {
        heap.push(value);
        let layout: Vec<_> = heap.iter().copied().collect();
        for child in 1..layout.len() {
            assert!(layout[(child - 1) / 2] >= layout[child]);
        }
    }

    let mut removed = Vec::new();
    while let Some(value) = heap.pop() {
        removed.push(value);
        let layout: Vec<_> = heap.iter().copied().collect();
        for child in 1..layout.len() {
            assert!(layout[(child - 1) / 2] >= layout[child]);
        }
    }
    assert_eq!(removed, [9, 9, 7, 4, 2, 1]);
}

#[test]
fn binary_heap_builds_from_values() {
    let mut heap = BinaryHeap::from(vec![3, 1, 8, 2]);
    assert_eq!(heap.peek(), Some(&8));
    assert_eq!(heap.pop(), Some(8));
    assert_eq!(heap.pop(), Some(3));
}

#[test]
fn priority_queue_orders_by_priority_without_ordering_values() {
    #[derive(Debug, PartialEq)]
    struct Job(&'static str);

    let mut jobs = PriorityQueue::new();
    assert_eq!(jobs.pop(), None);
    jobs.push(1, Job("background"));
    jobs.push(5, Job("urgent"));
    jobs.push(3, Job("normal"));

    assert_eq!(jobs.peek().map(|(priority, _)| *priority), Some(5));
    assert_eq!(jobs.pop(), Some((5, Job("urgent"))));
    assert_eq!(jobs.pop(), Some((3, Job("normal"))));
    assert_eq!(jobs.pop(), Some((1, Job("background"))));
    assert!(jobs.is_empty());
}

#[test]
fn binary_search_tree_rejects_duplicates_and_iterates_in_order() {
    let mut tree = BinarySearchTree::new();
    for value in [8, 3, 10, 1, 6, 14, 4, 7, 13] {
        assert!(tree.insert(value));
    }
    assert!(!tree.insert(6));
    assert_eq!(tree.len(), 9);
    assert!(tree.contains(&13));
    assert!(!tree.contains(&2));
    assert_eq!(tree.get(&6), Some(&6));
    assert_eq!(
        tree.iter().copied().collect::<Vec<_>>(),
        [1, 3, 4, 6, 7, 8, 10, 13, 14]
    );
}

#[test]
fn binary_search_tree_removes_leaf_one_child_two_children_and_root() {
    let mut tree: BinarySearchTree<_> = [8, 3, 10, 1, 6, 14, 4, 7, 13].into_iter().collect();

    assert_eq!(tree.remove(&1), Some(1)); // leaf
    assert_eq!(tree.remove(&14), Some(14)); // one child
    assert_eq!(tree.remove(&3), Some(3)); // two children
    assert_eq!(tree.remove(&8), Some(8)); // root with two children
    assert_eq!(tree.remove(&99), None);
    assert_eq!(tree.iter().copied().collect::<Vec<_>>(), [4, 6, 7, 10, 13]);
    assert_eq!(tree.len(), 5);
}

#[test]
fn binary_search_tree_clear_resets_root_and_length() {
    let mut tree: BinarySearchTree<_> = (0..100).collect();
    tree.clear();
    assert!(tree.is_empty());
    assert_eq!(tree.iter().next(), None);
    assert!(tree.insert(5));
    assert_eq!(tree.iter().copied().collect::<Vec<_>>(), [5]);
}

#[test]
fn binary_search_tree_consuming_iterator_is_in_order() {
    let tree: BinarySearchTree<_> = [5, 2, 8, 1, 3].into_iter().collect();
    assert_eq!(tree.into_iter().collect::<Vec<_>>(), [1, 2, 3, 5, 8]);
}

#[test]
fn recursive_and_iterative_search_trees_agree() {
    let mut iterative = BinarySearchTree::new();
    let mut recursive = RecursiveBinarySearchTree::new();

    for value in [8, 3, 10, 1, 6, 14, 4, 7, 13, 6] {
        assert_eq!(iterative.insert(value), recursive.insert(value));
    }
    for value in [6, 14, 8, 42, 3] {
        assert_eq!(iterative.contains(&value), recursive.contains(&value));
        assert_eq!(iterative.remove(&value), recursive.remove(&value));
        assert_eq!(
            iterative.iter().copied().collect::<Vec<_>>(),
            recursive.iter().copied().collect::<Vec<_>>()
        );
    }
}

#[test]
fn avl_tree_rejects_duplicates_and_stays_sorted_after_removal() {
    let mut tree: AvlTree<_> = [9, 5, 10, 0, 6, 11, -1, 1, 2].into_iter().collect();
    assert!(!tree.insert(6));
    assert_eq!(tree.get(&10), Some(&10));
    assert_eq!(tree.remove(&10), Some(10));
    assert_eq!(tree.remove(&42), None);
    assert_eq!(
        tree.iter().copied().collect::<Vec<_>>(),
        [-1, 0, 1, 2, 5, 6, 9, 11]
    );
    assert!(tree.height() <= 4);
}

#[test]
fn trie_handles_empty_unicode_prefixes_and_prunes_removed_words() {
    let mut trie = Trie::new();
    assert!(!trie.starts_with(""));
    assert!(trie.insert(""));
    assert!(trie.insert("car"));
    assert!(trie.insert("cart"));
    assert!(trie.insert("猫咪"));
    assert!(!trie.insert("car"));

    assert!(trie.contains(""));
    assert!(!trie.contains("ca"));
    assert!(trie.starts_with("ca"));
    assert!(trie.starts_with("猫"));
    assert!(trie.remove("car"));
    assert!(trie.starts_with("car")); // `cart` still uses the path
    assert!(trie.remove("cart"));
    assert!(!trie.starts_with("car")); // the unused path was pruned
    assert_eq!(trie.iter().collect::<Vec<_>>(), ["", "猫咪"]);
    assert_eq!(trie.len(), 2);
}

#[test]
fn red_black_tree_provides_ordered_unique_set_operations() {
    let mut tree: RedBlackTree<_> = [8, 3, 10, 1, 6, 14, 4, 7, 13].into_iter().collect();
    assert!(!tree.insert(6));
    assert!(tree.contains(&13));
    assert_eq!(tree.remove(&3), Some(3));
    assert_eq!(tree.remove(&99), None);
    assert_eq!(
        tree.iter().copied().collect::<Vec<_>>(),
        [1, 4, 6, 7, 8, 10, 13, 14]
    );
}

#[test]
fn b_tree_map_splits_updates_deletes_and_iterates_in_key_order() {
    let mut map = BTreeMap::new();
    for key in (0..40).rev() {
        assert_eq!(map.insert(key, key * 10), None);
    }
    assert!((2..=5).contains(&map.height()));
    assert_eq!(map.insert(12, 999), Some(120));
    assert_eq!(map.get(&12), Some(&999));
    *map.get_mut(&13).unwrap() = 777;
    assert_eq!(map.remove(&14), Some(140));
    assert_eq!(map.remove(&99), None);
    assert_eq!(
        map.iter().map(|(key, _)| *key).collect::<Vec<_>>(),
        (0..40).filter(|key| *key != 14).collect::<Vec<_>>()
    );
    assert_eq!(map.get(&13), Some(&777));
}

#[test]
fn b_tree_set_keeps_unique_values_in_order() {
    let mut set = BTreeSet::new();
    assert!(set.insert(4));
    assert!(set.insert(1));
    assert!(!set.insert(4));
    assert!(set.contains(&1));
    assert!(set.remove(&4));
    assert!(!set.remove(&4));
    assert_eq!(set.iter().copied().collect::<Vec<_>>(), [1]);
}

fn verify_tree_map<M: TreeMap<i32, String>>(mut map: M) {
    assert!(map.is_empty());
    assert_eq!(map.insert(3, "three".into()), None);
    assert_eq!(map.insert(1, "one".into()), None);
    assert_eq!(map.insert(2, "two".into()), None);
    assert_eq!(map.insert(2, "TWO".into()), Some("two".into()));
    *map.get_mut(&3).unwrap() = "THREE".into();
    assert!(map.contains_key(&1));
    assert_eq!(map.get(&2).map(String::as_str), Some("TWO"));
    assert_eq!(
        map.iter()
            .map(|(k, v)| (*k, v.as_str()))
            .collect::<Vec<_>>(),
        [(1, "one"), (2, "TWO"), (3, "THREE")]
    );
    assert_eq!(map.remove(&2).as_deref(), Some("TWO"));
    assert_eq!(map.remove(&2), None);
    assert_eq!(map.len(), 2);
}

#[test]
fn tree_map_backends_share_ordered_map_behavior() {
    verify_tree_map(BinarySearchTreeMap::new());
    verify_tree_map(RecursiveBinarySearchTreeMap::new());
    verify_tree_map(AvlTreeMap::new());
    verify_tree_map(RedBlackTreeMap::new());
    verify_tree_map(BTreeMap::new());
    verify_tree_map(BPlusTreeMap::new());
}

#[test]
fn b_plus_tree_set_keeps_unique_values_in_order() {
    let mut set = BPlusTreeSet::new();
    for value in (0..24).rev() {
        assert!(set.insert(value));
    }
    assert!(!set.insert(12));
    assert!(set.remove(&12));
    assert!(!set.remove(&12));
    assert_eq!(
        set.iter().copied().collect::<Vec<_>>(),
        (0..24).filter(|value| *value != 12).collect::<Vec<_>>()
    );
}

#[test]
fn b_plus_tree_map_supports_borrowed_string_keys() {
    let mut map = BPlusTreeMap::new();
    map.insert(String::from("pear"), 4);
    map.insert(String::from("apple"), 2);
    assert_eq!(map.get("apple"), Some(&2));
    assert_eq!(map.remove("pear"), Some(4));
    map.clear();
    assert!(map.is_empty());
}

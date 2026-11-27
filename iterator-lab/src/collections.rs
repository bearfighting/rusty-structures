use data_structures::{
    AvlTree, AvlTreeMap, BPlusTreeMap, BPlusTreeSet, BTreeMap, BTreeSet, BinarySearchTree,
    BinarySearchTreeMap, BitSet, HashMap, HashSet, PersistentAvlTreeMap, PersistentTrie, RadixTrie,
    RecursiveBinarySearchTree, RecursiveBinarySearchTreeMap, RedBlackTree, RedBlackTreeMap,
    TreeMap, Trie,
};

fn export_incidents<M: TreeMap<u32, &'static str>>(map: &M) -> Vec<(u32, &'static str)> {
    map.iter().map(|(&id, &message)| (id, message)).collect()
}

pub fn run() {
    // Search-tree set variants keep unique identifiers and emit sorted output.
    let mut import_batch = BinarySearchTree::new();
    let mut recursive_batch = RecursiveBinarySearchTree::new();
    for id in [7304, 7288, 7304, 7311] {
        import_batch.insert(id);
        recursive_batch.insert(id);
    }
    println!(
        "BST import IDs: {:?}",
        import_batch.iter().collect::<Vec<_>>()
    );
    println!(
        "recursive BST agrees: {}",
        import_batch.iter().eq(recursive_batch.iter())
    );

    // The map wrappers share a trait, so the same task can use an unbalanced,
    // recursive, AVL, or red-black backend.
    let mut iterative_map = BinarySearchTreeMap::new();
    let mut recursive_map = RecursiveBinarySearchTreeMap::new();
    for (id, message) in [(7304, "queued"), (7288, "triaged"), (7311, "closed")] {
        iterative_map.insert(id, message);
        recursive_map.insert(id, message);
    }
    println!(
        "iterative map export: {:?}",
        export_incidents(&iterative_map)
    );
    println!(
        "recursive map export: {:?}",
        export_incidents(&recursive_map)
    );

    let mut partitions = AvlTree::new();
    for id in [18, 4, 27, 12, 18] {
        partitions.insert(id);
    }
    partitions.remove(&18);
    println!(
        "active partitions: {:?}",
        partitions.iter().collect::<Vec<_>>()
    );

    let mut pending = RedBlackTree::new();
    for id in [4102, 3988, 4271, 4102] {
        pending.insert(id);
    }
    println!(
        "pending review IDs: {:?}",
        pending.iter().collect::<Vec<_>>()
    );

    // Fold a sequence into a frequency map. Hash-map iteration is arbitrary,
    // so sort before presenting results when deterministic output is needed.
    let words = ["rust", "data", "rust", "borrow", "data", "rust"];
    let mut counts = HashMap::new();
    for word in words {
        let count = counts.get(word).copied().unwrap_or(0usize);
        counts.insert(word, count + 1);
    }
    let mut frequencies: Vec<_> = counts.iter().map(|(&word, &n)| (word, n)).collect();
    frequencies.sort_unstable();
    println!("word frequencies: {frequencies:?}");

    let unique: HashSet<_> = words.into_iter().collect();
    println!("unique words: {}", unique.len());

    let mut hits: HashMap<String, usize> = HashMap::with_capacity(8);
    for path in ["/", "/api/items", "/", "/api/items", "/login"] {
        let next = hits.get(path).copied().unwrap_or(0) + 1;
        hits.insert(path.to_owned(), next);
    }
    let mut report: Vec<_> = hits
        .iter()
        .map(|(path, &count)| (path.as_str(), count))
        .collect();
    report.sort_unstable_by(|left, right| left.0.cmp(right.0));
    println!("request report: {report:?}");

    // Ordered maps and tries promise a meaningful traversal order.
    let mut grades = BTreeMap::new();
    grades.insert("Mina", 91);
    grades.insert("Ada", 98);
    grades.insert("Linus", 87);
    let honors: Vec<_> = grades
        .iter()
        .filter_map(|(name, &grade)| (grade >= 90).then_some((*name, grade)))
        .collect();
    println!("honors, by key: {honors:?}");

    // An AVL map keeps a frequently updated index balanced and emits records
    // in key order. Filtering this full iterator is O(n), as there is no range
    // cursor in the current TreeMap API.
    let mut stock = AvlTreeMap::new();
    stock.insert(104, 7);
    stock.insert(205, 2);
    stock.insert(318, 0);
    stock.insert(421, 4);
    let restock: Vec<_> = stock
        .iter()
        .filter_map(|(&sku, &quantity)| (quantity < 5).then_some((sku, quantity)))
        .collect();
    println!("ordered restock report: {restock:?}");

    let mut incidents = RedBlackTreeMap::new();
    incidents.insert(4102, "pool exhausted");
    incidents.insert(3988, "payment latency");
    incidents.insert(4271, "disk pressure");
    incidents.insert(4102, "pool recovered");
    println!(
        "incident audit order: {:?}",
        incidents
            .iter()
            .map(|(&id, &message)| (id, message))
            .collect::<Vec<_>>()
    );

    // B+ tree iteration follows the linked leaf layer. `take` stops the scan
    // as soon as the report has enough records.
    let mut timeline = BPlusTreeMap::new();
    timeline.insert(9, "standup");
    timeline.insert(11, "review");
    timeline.insert(14, "release");
    let next_events: Vec<_> = timeline
        .iter()
        .take(2)
        .map(|(&hour, &event)| (hour, event))
        .collect();
    println!("next events: {next_events:?}");

    let mut deployed = BTreeSet::new();
    for version in [104, 101, 104, 108, 101] {
        deployed.insert(version);
    }
    println!(
        "unique deployment IDs: {:?}",
        deployed.iter().collect::<Vec<_>>()
    );
    let mut scanned_versions = BPlusTreeSet::new();
    for version in [104, 101, 104, 108, 101] {
        scanned_versions.insert(version);
    }
    println!(
        "B+ set versions: {:?}",
        scanned_versions.iter().collect::<Vec<_>>()
    );

    let mut trie = Trie::new();
    for word in ["car", "cart", "cat", "dog"] {
        trie.insert(word);
    }
    println!(
        "all words in trie order: {:?}",
        trie.iter().collect::<Vec<_>>()
    );
    println!(
        "`ca` suggestions: {:?}",
        trie.words_with_prefix("ca").collect::<Vec<_>>()
    );

    let mut routes = RadixTrie::new();
    routes.insert("/api/v1/users");
    routes.insert("/api/v1/orders");
    println!("registered API prefix: {}", routes.starts_with("/api/v1"));

    let stable: PersistentTrie = ["build", "check", "test"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    let (preview, _) = stable.insert("bench");
    println!(
        "stable has bench: {}; preview has bench: {}",
        stable.contains("bench"),
        preview.contains("bench")
    );

    let (config, _) = PersistentAvlTreeMap::new().insert("timeout_ms", 800);
    let (preview_config, _) = config.insert("timeout_ms", 1_200);
    println!(
        "config snapshots: {:?} -> {:?}",
        config.get("timeout_ms"),
        preview_config.get("timeout_ms")
    );

    // BitSet iteration enumerates the set members, not every possible bit.
    let mut visited = BitSet::new();
    for node in [2, 5, 8, 13] {
        visited.insert(node);
    }
    println!("visited node IDs: {:?}", visited.iter().collect::<Vec<_>>());
}

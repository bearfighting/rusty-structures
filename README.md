# Rust 数据结构学习集合

这是一个用 Rust 实现经典数据结构的学习型项目，目标是让代码易读、易懂，同时对关键不变量、边界行为和复杂度保持严谨。项目适合阅读、比较不同实现方式，并练习 Rust 的所有权与借用。

本项目不定位为 Rust 标准库的替代品。实现以讲清楚核心算法和设计取舍为先，不承诺与标准库相同的性能。

## 已实现

- 线性容器：`Vector`、`Stack`、`Queue`、`Deque`、`FixedCapacityQueue`
- 链表：`SinglyLinkedList`、`DoublyLinkedList`
- 哈希结构：`HashMap`、`HashSet`
- 概率型集合：`BloomFilter`，以可配置的误报率换取紧凑的成员查询。
- 整数集合：`BitSet`，用紧凑位数组维护非负整数集合并支持集合运算。
- 缓存：`LruCache`，使用固定容量和最近使用顺序淘汰条目。
- 优先结构：`BinaryHeap`、`DaryHeap`、`PriorityQueue`、`IndexPriorityQueue`
- 树与前缀结构：`BinarySearchTree`（迭代版）、`RecursiveBinarySearchTree`、`AvlTree`、`RedBlackTree`、`BTreeMap`、`BTreeSet`、`BPlusTreeMap`、`BPlusTreeSet`、`Trie`、`RadixTrie`、`PersistentTrie`、`PersistentAvlTreeMap`
- 概率有序结构：`SkipListMap`、`SkipListSet`
- 有序映射：`TreeMap` trait，以及由二叉搜索树、AVL 树、红黑树、B 树和 B+ 树支撑的映射实现。它们共享映射行为接口，便于比较不同树的算法和不变量。
- 图：邻接表 `Graph` / `WeightedGraph` 与矩阵 `AdjacencyMatrixGraph` / `WeightedAdjacencyMatrixGraph`，支持有向/无向图、DFS/BFS、连通分量、拓扑排序、无权最短路径和 Dijkstra 带权最短路径。
- 区间与查询：`DisjointSet`（路径压缩、按大小合并）、`FenwickTree`（单点增量与区间和）、泛型 `SegmentTree`（基于 `Monoid` 的单点赋值与区间查询）、`LazySegmentTree`（通过规格 trait 支持区间更新与查询）、`SparseTable`（静态幂等区间查询）。

结构的操作语义、复杂度和示例见各自的 Rustdoc；模块分层与 algebra 合约见 [ARCHITECTURE.md](ARCHITECTURE.md)，推进计划见 [ROADMAP.md](ROADMAP.md)，设计约定见 [PROJECT_PLAN.md](PROJECT_PLAN.md)。

教材的迭代器综合章配有独立可运行的例子：在仓库根目录执行 `cargo run -p iterator-lab`，或阅读 [`iterator-lab/README.md`](iterator-lab/README.md) 按主题查看。

数据结构可视化教学工具的原型设计见 [VISUALIZER_DESIGN.md](VISUALIZER_DESIGN.md)，分阶段交付计划见 [VISUALIZER_ROADMAP.md](VISUALIZER_ROADMAP.md)。

## 验证

```sh
cargo test
cargo fmt --check
```

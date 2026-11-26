# Rust 数据结构学习集合

这是一个用 Rust 实现经典数据结构的学习型项目，目标是让代码易读、易懂，同时对关键不变量、边界行为和复杂度保持严谨。项目适合阅读、比较不同实现方式，并练习 Rust 的所有权与借用。

本项目不定位为 Rust 标准库的替代品。实现以讲清楚核心算法和设计取舍为先，不承诺与标准库相同的性能。

## 已实现

- 线性容器：`Vector`、`Stack`、`Queue`、`Deque`
- 链表：`SinglyLinkedList`、`DoublyLinkedList`
- 哈希结构：`HashMap`、`HashSet`
- 优先结构：`BinaryHeap`、`PriorityQueue`
- 树与前缀结构：`BinarySearchTree`（迭代版）、`RecursiveBinarySearchTree`、`AvlTree`、`RedBlackTree`、`BTreeMap`、`BTreeSet`、`BPlusTreeMap`、`BPlusTreeSet`、`Trie`
- 有序映射：`TreeMap` trait，以及由二叉搜索树、AVL 树、红黑树、B 树和 B+ 树支撑的映射实现。它们共享映射行为接口，便于比较不同树的算法和不变量。
- 图：邻接表 `Graph` / `WeightedGraph` 与矩阵 `AdjacencyMatrixGraph` / `WeightedAdjacencyMatrixGraph`，支持有向/无向图、DFS/BFS、连通分量、拓扑排序、无权最短路径和 Dijkstra 带权最短路径。
- 区间与查询：`DisjointSet`，使用路径压缩和按大小合并维护不相交集合。

结构的操作语义、复杂度和示例见各自的 Rustdoc；模块分层见 [ARCHITECTURE.md](ARCHITECTURE.md)，推进计划见 [ROADMAP.md](ROADMAP.md)，设计约定见 [PROJECT_PLAN.md](PROJECT_PLAN.md)。

数据结构可视化教学工具的原型设计见 [VISUALIZER_DESIGN.md](VISUALIZER_DESIGN.md)，分阶段交付计划见 [VISUALIZER_ROADMAP.md](VISUALIZER_ROADMAP.md)。

## 验证

```sh
cargo test
cargo fmt --check
```

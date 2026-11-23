# Rust 数据结构学习集合

这是一个用 Rust 实现经典数据结构的学习型项目，目标是让代码易读、易懂，同时对关键不变量、边界行为和复杂度保持严谨。项目适合阅读、比较不同实现方式，并练习 Rust 的所有权与借用。

本项目不定位为 Rust 标准库的替代品。实现以讲清楚核心算法和设计取舍为先，不承诺与标准库相同的性能。

## 已实现

- 线性容器：`Vector`、`Stack`、`Queue`、`Deque`
- 链表：`SinglyLinkedList`、`DoublyLinkedList`
- 哈希结构：`HashMap`、`HashSet`
- 优先结构：`BinaryHeap`、`PriorityQueue`
- 树：`AvlTree`、迭代版 `BinarySearchTree`、递归版 `RecursiveBinarySearchTree`、`Trie`

结构的操作语义、复杂度和示例见各自的 Rustdoc；推进计划见 [ROADMAP.md](ROADMAP.md)，设计约定见 [PROJECT_PLAN.md](PROJECT_PLAN.md)。

## 验证

```sh
cargo test
cargo fmt --check
```

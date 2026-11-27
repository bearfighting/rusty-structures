# Iterator Lab

这个 workspace package 收集数据结构与 Rust iterator 的组合示例。它面向教材读者，程序运行后会打印各示例的结果。

从仓库根目录运行完整练习场：

```sh
cargo run -p iterator-lab
```

## 示例地图

- `src/linear.rs`：Vector 的转换流水线、可变遍历、Stack/Queue 的抽象顺序、Deque 双端迭代与短路查找。
- `src/collections.rs`：BST/AVL/红黑树集合、AVL/红黑映射、B+ 时间表、B 树集合、Trie 命令候选、压缩路由前缀和持久化配置快照。
- `src/heaps.rs`：区分堆内部布局与按优先级反复弹出。
- `src/graphs.rs`：邻居迭代器和 BFS 结果的后续处理。
- `src/ranges.rs`：逐元素迭代与 SegmentTree 区间聚合的职责区别。

教材对应章节为 `book/chapters/iterators.qmd`。可在各模块里修改输入、替换适配器，观察结果和遍历顺序如何变化。

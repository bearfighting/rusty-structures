# 教材样稿构建

本目录以 Quarto Book 作为教材源稿，同时生成在线 HTML 和印刷版 PDF。PDF 可用 `--to typst`（Typst）或 `--to pdf`（LuaLaTeX）输出；两条路径都配置了中文字体。

## 本地构建

需要安装 Quarto CLI。Quarto 自带 Typst 编译器，因此不需要另装 Typst。PDF 样式使用 Noto CJK 字体；本开发环境已经安装配置中列出的字体族，其他系统可安装对应字体或调整 `_quarto.yml`。Typst 书籍输出目前用 `include-in-header` 显式设定正文字体，避免 Quarto 的书籍模板漏传 `mainfont`。

```sh
cd book
quarto preview
quarto render --to html
quarto render --to typst
quarto render --to pdf
```

Quarto 的 Typst 图书输出默认使用 orange-book 样式。样章完成审阅后，再决定是否制作项目专属的 Typst 格式扩展。

## 内容维护约定

- 章节代码示例应与 `src/` 的 Rustdoc 示例保持一致；当前 `Vector` 基础用法已在 `src/linear/vector.rs` 有可运行的 doctest。
- 跨结构的综合用法维护在 workspace package `iterator-lab/`，可执行 `cargo run -p iterator-lab`；教材的新示例应优先从这些程序挑选并保持同步。
- 结构状态图保存为 SVG，保留文本源文件，便于修订与出版前调整。
- 复杂度和边界行为必须与实现同步。修改 API 后，检查本章接口表、例子和源码导读。
- 生成目录 `_book/` 是构建产物，不在教材源文件中手工修改。


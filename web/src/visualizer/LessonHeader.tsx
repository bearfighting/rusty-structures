import LessonSelector from './LessonSelector';

export default function LessonHeader({ selectedId }: { selectedId: string }) {
  return <header className="topbar">
    <a className="brand" href="#lesson/binary-search-tree" aria-label="Data Structure Lab 首页">
      <span className="brand-mark" aria-hidden="true">DS</span>
      <span className="brand-copy"><strong>Data Structure Lab</strong><small>Rust implementations, made visible</small></span>
    </a>
    <nav className="topbar-controls" aria-label="课程导航">
      <div className="topbar-meta"><span className="live-dot" />交互式数据结构课堂</div>
      <LessonSelector selectedId={selectedId} />
      <a className="source-link" href="https://github.com/bearfighting/rusty-structures" target="_blank" rel="noreferrer">项目源码 ↗</a>
    </nav>
  </header>;
}

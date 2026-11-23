import { useEffect, useMemo, useReducer, useRef } from 'react';
import init, { BstSession } from './generated/data_structures_visualizer_wasm';
import type { NodeView, TraceEvent, TraceView, TreeView } from './types';

type Operation = 'insert' | 'search' | 'remove';
type State = {
  ready: boolean;
  error: string | null;
  operation: Operation;
  input: string;
  tree: TreeView;
  trace: TraceView | null;
  cursor: number;
  playing: boolean;
  speed: number;
  selected: string | null;
};
type Action =
  | { type: 'ready'; tree: TreeView }
  | { type: 'error'; message: string }
  | { type: 'input'; value: string }
  | { type: 'operation'; value: Operation }
  | { type: 'speed'; value: number }
  | { type: 'run'; trace: TraceView }
  | { type: 'cursor'; value: number }
  | { type: 'playing'; value: boolean }
  | { type: 'tree'; value: TreeView }
  | { type: 'selected'; value: string | null }
  | { type: 'reset'; tree: TreeView };

const emptyTree: TreeView = { root: null, len: 0 };
const initialState: State = {
  ready: false, error: null, operation: 'insert', input: '', tree: emptyTree,
  trace: null, cursor: 0, playing: false, speed: 760, selected: null,
};

function reducer(state: State, action: Action): State {
  switch (action.type) {
    case 'ready': return { ...state, ready: true, tree: action.tree, error: null };
    case 'error': return { ...state, error: action.message };
    case 'input': return { ...state, input: action.value };
    case 'operation': return { ...state, operation: action.value };
    case 'speed': return { ...state, speed: action.value };
    case 'run': return { ...state, trace: action.trace, cursor: 0, playing: false, selected: null, error: null };
    case 'cursor': {
      const end = state.trace?.events.length ?? 0;
      const cursor = Math.max(0, Math.min(end, action.value));
      return { ...state, cursor, playing: cursor < end ? state.playing : false };
    }
    case 'playing': return { ...state, playing: action.value };
    case 'tree': return { ...state, tree: action.value, trace: null, cursor: 0, playing: false, selected: null };
    case 'selected': return { ...state, selected: action.value };
    case 'reset': return { ...state, tree: action.tree, trace: null, cursor: 0, playing: false, selected: null, error: null };
  }
}

export default function App() {
  const [state, dispatch] = useReducer(reducer, initialState);
  const session = useRef<BstSession | null>(null);
  const busy = Boolean(state.trace && state.cursor < state.trace.events.length);
  const tree = state.trace && state.cursor >= state.trace.events.length ? state.trace.after : state.trace?.before ?? state.tree;
  const event = state.trace?.events[Math.min(state.cursor, Math.max(0, state.trace.events.length - 1))] ?? null;
  const activeIds = useMemo(() => getActiveIds(state.trace, state.cursor), [state.trace, state.cursor]);
  const invariant = useMemo(() => checkTree(tree), [tree]);

  useEffect(() => {
    let active = true;
    void init().then(() => {
      if (!active) return;
      const bst = new BstSession();
      session.current = bst;
      dispatch({ type: 'ready', tree: bst.load_example() });
    }).catch((error: unknown) => {
      dispatch({ type: 'error', message: `Wasm 加载失败：${String(error)}` });
    });
    return () => { active = false; };
  }, []);

  useEffect(() => {
    if (!state.playing) return;
    let frame = 0;
    let previous = 0;
    let elapsed = 0;
    const tick = (now: number) => {
      if (previous) elapsed += now - previous;
      previous = now;
      if (elapsed >= state.speed) {
        elapsed = 0;
        dispatch({ type: 'cursor', value: state.cursor + 1 });
      }
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame);
  }, [state.playing, state.cursor, state.speed]);

  function run() {
    const key = Number(state.input);
    if (!Number.isSafeInteger(key)) {
      dispatch({ type: 'error', message: '请输入一个安全范围内的整数。' });
      return;
    }
    if (!session.current) return;
    try {
      const trace = session.current[state.operation](key);
      dispatch({ type: 'run', trace });
    } catch (error) {
      dispatch({ type: 'error', message: String(error) });
    }
  }

  function reset() {
    if (!session.current) return;
    dispatch({ type: 'reset', tree: session.current.reset() });
  }

  function loadExample() {
    if (!session.current) return;
    dispatch({ type: 'reset', tree: session.current.load_example() });
  }

  function stepForward() { dispatch({ type: 'cursor', value: state.cursor + 1 }); }
  function stepBack() { dispatch({ type: 'cursor', value: state.cursor - 1 }); }

  return (
    <main className="app-shell">
      <header className="topbar">
        <a className="brand" href="#top" aria-label="Tree Lab 首页">
          <span className="brand-mark" aria-hidden="true">T</span>
          <span>tree<span className="brand-light">lab</span></span>
        </a>
        <div className="topbar-meta"><span className="live-dot" />交互式数据结构课堂</div>
        <a className="source-link" href="https://github.com/bearfighting/rusty-structures" target="_blank" rel="noreferrer">项目源码 ↗</a>
      </header>

      <section className="intro" id="top">
        <div className="eyebrow"><span>LESSON 01</span><i />二叉搜索树</div>
        <h1>每一次比较，<em>都看得见。</em></h1>
        <p>亲手执行插入、查找和删除。放慢算法，一步一步看清树如何生长。</p>
      </section>

      <section className="workspace" aria-label="二叉搜索树交互区">
        <div className="canvas-card">
          <div className="card-heading">
            <div><span className="section-kicker">TREE CANVAS</span><h2>当前结构</h2></div>
            <div className="tree-count"><strong>{tree.len}</strong><span>个节点</span><i />高度 <strong>{treeHeight(tree.root)}</strong></div>
          </div>
          <div className="tree-canvas">
            {tree.root ? <TreeCanvas tree={tree} activeIds={activeIds} selected={state.selected} onSelect={(id) => dispatch({ type: 'selected', value: id })} /> : (
              <div className="empty-canvas"><div className="empty-glyph">∅</div><strong>树是空的</strong><span>插入一个整数，开始构建你的树。</span></div>
            )}
          </div>
          <div className="canvas-legend"><span><i className="legend-node" />节点</span><span><i className="legend-node active" />当前比较</span><span><i className="legend-edge" />搜索路径</span></div>
        </div>

        <aside className="side-column">
          <section className="panel operation-panel">
            <div className="panel-title"><span className="section-kicker">TRY IT YOURSELF</span><h2>执行操作</h2></div>
            <label className="field-label" htmlFor="operation">操作</label>
            <select id="operation" value={state.operation} onChange={(event) => dispatch({ type: 'operation', value: event.target.value as Operation })} disabled={!state.ready || busy}>
              <option value="insert">插入节点</option><option value="search">查找节点</option><option value="remove">删除节点</option>
            </select>
            <label className="field-label" htmlFor="key-input">整数键</label>
            <div className="input-row">
              <input id="key-input" type="number" step="1" value={state.input} onChange={(event) => dispatch({ type: 'input', value: event.target.value })} onKeyDown={(event) => { if (event.key === 'Enter' && !busy) run(); }} placeholder="例如 7" disabled={!state.ready || busy} />
              <button className="primary-button" onClick={run} disabled={!state.ready || busy}>{busy ? '逐步播放中' : '运行操作'}<span aria-hidden="true">↗</span></button>
            </div>
            <div className="button-row"><button className="text-button" onClick={loadExample} disabled={!state.ready || busy}>载入示例树</button><button className="text-button" onClick={reset} disabled={!state.ready}>清空</button></div>
            {state.error && <p className="error-message" role="alert">{state.error}</p>}
            <div className="divider" />
            <div className="playback-heading"><span className="section-kicker">PLAYBACK</span><span>{state.trace ? `${state.cursor} / ${state.trace.events.length}` : '等待操作'}</span></div>
            <div className="playback-controls">
              <button className="icon-button" onClick={stepBack} disabled={!state.trace || state.cursor === 0} aria-label="上一步">⟵</button>
              <button className="play-button" onClick={() => dispatch({ type: 'playing', value: !state.playing })} disabled={!state.trace || (!state.playing && state.cursor >= state.trace.events.length)} aria-label={state.playing ? '暂停' : '播放'}>{state.playing ? 'Ⅱ' : '▶'}</button>
              <button className="icon-button" onClick={stepForward} disabled={!state.trace || state.cursor >= state.trace.events.length} aria-label="下一步">⟶</button>
              <button className="icon-button reset-icon" onClick={() => state.trace && dispatch({ type: 'cursor', value: 0 })} disabled={!state.trace || state.cursor === 0} aria-label="回到开始">↺</button>
              <label className="speed-control"><span>速度</span><input aria-label="播放速度" type="range" min="250" max="1500" step="50" value={1750 - state.speed} onChange={(event) => dispatch({ type: 'speed', value: 1750 - Number(event.target.value) })} /><span>{state.speed < 500 ? '快' : state.speed > 1050 ? '慢' : '中'}</span></label>
            </div>
          </section>

          <section className="panel lesson-panel" aria-live="polite">
            <div className="panel-title compact"><span className="section-kicker">ALGORITHM STEP</span><span className="step-number">{state.trace ? `${Math.min(state.cursor + 1, state.trace.events.length)} / ${state.trace.events.length}` : '—'}</span></div>
            <h2>{eventTitle(event, state.trace, state.cursor)}</h2>
            <p className="step-explanation">{eventExplanation(event, state.trace, state.cursor)}</p>
            <div className="pseudocode-title">伪代码 <span>BST OPERATIONS</span></div>
            <ol className="pseudocode">{pseudoLines(state.operation).map((line, index) => <li key={line} className={index === pseudoLine(event) ? 'current' : ''}><code>{line}</code></li>)}</ol>
            <div className={`invariant ${invariant.ok ? 'ok' : 'bad'}`}><span className="invariant-icon">{invariant.ok ? '✓' : '!'}</span><div><strong>{invariant.ok ? 'BST 顺序不变量成立' : '检测到顺序错误'}</strong><span>{invariant.ok ? `已检查 ${invariant.count} 个节点，左小右大。` : invariant.message}</span></div></div>
            {state.selected && <div className="node-inspector"><span className="section-kicker">SELECTED NODE</span><strong>{state.selected}</strong><span>{nodeRelation(tree.root, state.selected)}</span></div>}
          </section>
        </aside>
      </section>

      <section className="timeline-card" aria-label="操作步骤时间线">
        <div className="timeline-heading"><div><span className="section-kicker">OPERATION TRACE</span><h2>算法步骤</h2></div><span className="timeline-hint">选择步骤可跳转</span></div>
        {state.trace?.events.length ? <ol className="event-list">{state.trace.events.map((step, index) => <li key={`${index}-${step.kind}`}><button className={index === Math.min(state.cursor, state.trace!.events.length - 1) ? 'selected' : ''} onClick={() => dispatch({ type: 'cursor', value: index })}><span className="event-index">{String(index + 1).padStart(2, '0')}</span><span>{shortEvent(step)}</span><i>{index < state.cursor ? '✓' : '·'}</i></button></li>)}</ol> : <div className="timeline-empty">执行一次操作，这里会显示 Rust BST 实际经过的比较与分支。</div>}
      </section>

      <footer><span>由 Rust BST 驱动</span><span>Tree Lab · 原型 0.1</span><span>学习算法，从观察开始。</span></footer>
    </main>
  );
}

function TreeCanvas({ tree, activeIds, selected, onSelect }: { tree: TreeView; activeIds: Set<string>; selected: string | null; onSelect: (id: string) => void }) {
  const layout = useMemo(() => layoutTree(tree.root), [tree.root]);
  if (!tree.root) return null;
  return <svg className="tree-svg" viewBox={`0 0 ${layout.width} ${layout.height}`} role="img" aria-label={`二叉搜索树，共 ${tree.len} 个节点`}>
    <g className="edges">{layout.edges.map((edge) => <line key={edge.key} x1={edge.x1} y1={edge.y1} x2={edge.x2} y2={edge.y2} className={activeIds.has(edge.parent) && activeIds.has(edge.child) ? 'path-edge' : ''} />)}</g>
    <g className="nodes">{layout.nodes.map((node) => <g key={node.id} className={`tree-node ${activeIds.has(node.id) ? 'active' : ''} ${selected === node.id ? 'selected' : ''}`} transform={`translate(${node.x}, ${node.y})`} onClick={() => onSelect(node.id)} onKeyDown={(event) => { if (event.key === 'Enter' || event.key === ' ') onSelect(node.id); }} role="button" tabIndex={0} aria-label={`节点 ${node.value}`}>
      <circle r="26" /><text textAnchor="middle" dominantBaseline="central">{node.value}</text>
    </g>)}</g>
    <text className="root-label" x={layout.nodes.find((node) => node.id === tree.root?.id)?.x} y="22" textAnchor="middle">ROOT</text>
  </svg>;
}

type Point = { id: string; value: number; x: number; y: number };
type Edge = { key: string; parent: string; child: string; x1: number; y1: number; x2: number; y2: number };
function layoutTree(root: NodeView | null) {
  const nodes: Point[] = [];
  const edges: Edge[] = [];
  let rank = 0;
  let maxDepth = 0;
  function visit(node: NodeView | null, depth: number): Point | null {
    if (!node) return null;
    const left = visit(node.left, depth + 1);
    const point = { id: node.id, value: node.value, x: 72 + rank * 104, y: 62 + depth * 104 };
    rank += 1;
    maxDepth = Math.max(maxDepth, depth);
    nodes.push(point);
    visit(node.right, depth + 1);
    return left ?? point;
  }
  visit(root, 0);
  const byId = new Map(nodes.map((node) => [node.id, node]));
  function connect(node: NodeView | null) {
    if (!node) return;
    const parent = byId.get(node.id)!;
    for (const child of [node.left, node.right]) {
      if (!child) continue;
      const point = byId.get(child.id)!;
      edges.push({ key: `${node.id}-${child.id}`, parent: node.id, child: child.id, x1: parent.x, y1: parent.y + 26, x2: point.x, y2: point.y - 26 });
      connect(child);
    }
  }
  connect(root);
  return { nodes, edges, width: Math.max(520, 144 + Math.max(0, rank - 1) * 104), height: Math.max(180, 108 + maxDepth * 104) };
}

function getActiveIds(trace: TraceView | null, cursor: number) {
  const ids = new Set<string>();
  if (!trace) return ids;
  const through = Math.min(cursor, trace.events.length);
  for (const event of trace.events.slice(0, Math.max(through, 1))) {
    if (event.kind === 'compare') ids.add(event.nodeId);
    if (event.kind === 'descend') ids.add(event.fromId);
    if (event.kind === 'inserted') ids.add(event.nodeId);
    if (event.kind === 'successorSelected') { ids.add(String(event.removed)); ids.add(String(event.successor)); }
  }
  return ids;
}

function eventTitle(event: TraceEvent | null, trace: TraceView | null, cursor: number) {
  if (!trace) return '准备开始';
  if (!trace.events.length) return '操作已完成';
  if (cursor >= trace.events.length) return '操作已完成';
  switch (event?.kind) {
    case 'compare': return `比较 ${event.key} 和 ${event.nodeValue}`;
    case 'descend': return event.direction === 'left' ? '向左子树继续' : '向右子树继续';
    case 'inserted': return `插入节点 ${event.key}`;
    case 'duplicate': return `节点 ${event.key} 已存在`;
    case 'found': return `找到节点 ${event.key}`;
    case 'missing': return `没有找到 ${event.key}`;
    case 'successorSelected': return `选择后继节点 ${event.successor}`;
    case 'removed': return `移除节点 ${event.key}`;
    default: return '操作已完成';
  }
}

function eventExplanation(event: TraceEvent | null, trace: TraceView | null, cursor: number) {
  if (!trace) return '选择一个操作并输入整数。每一步都会由 Rust 中的二叉搜索树实际执行。';
  if (!trace.events.length || cursor >= trace.events.length) return `操作结束。当前树包含 ${trace.after.len} 个节点，画布显示 Rust 返回的最终结构。`;
  switch (event?.kind) {
    case 'compare':
      return event.ordering === 'less' ? `${event.key} 小于当前节点 ${event.nodeValue}，按照 BST 规则转向左子树。` : event.ordering === 'greater' ? `${event.key} 大于当前节点 ${event.nodeValue}，按照 BST 规则转向右子树。` : `键 ${event.key} 与当前节点相等。查找已命中；插入会报告重复键。`;
    case 'descend': return `从节点 ${event.fromId} 沿${event.direction === 'left' ? '左' : '右'}链接继续查找。`;
    case 'inserted': return `目标链接为空，Rust 实现把 ${event.key} 放在这里，树大小增加一。`;
    case 'duplicate': return `树中已经有 ${event.key}。集合语义不插入重复值，结构保持不变。`;
    case 'found': return `找到 ${event.key}，查找路径到此结束。`;
    case 'missing': return `走到空链接仍未命中，${event.key} 不在树中。`;
    case 'successorSelected': return `节点 ${event.removed} 有两个孩子；先选右子树中最小的节点 ${event.successor} 作为中序后继。`;
    case 'removed': return `节点 ${event.key} 已移除，剩余子树接回原链接。`;
    default: return '观察当前操作路径。';
  }
}

function shortEvent(event: TraceEvent) {
  switch (event.kind) {
    case 'compare': return `比较 ${event.key} 与 ${event.nodeValue}`;
    case 'descend': return `进入${event.direction === 'left' ? '左' : '右'}子树`;
    case 'inserted': return `插入 ${event.key}`;
    case 'duplicate': return `${event.key} 已存在`;
    case 'found': return `找到 ${event.key}`;
    case 'missing': return `未找到 ${event.key}`;
    case 'successorSelected': return `后继：${event.successor}`;
    case 'removed': return `移除 ${event.key}`;
  }
}

function pseudoLines(operation: Operation) {
  if (operation === 'search') return ['current = root', '比较 key 与 current.value', '若 key 更小，进入 left', '若 key 更大，进入 right', '相等则返回 found', '空链接则返回 missing'];
  if (operation === 'remove') return ['current = root', '比较 key 与 current.value', '按大小继续向下查找', '0 或 1 个孩子：接回子树', '2 个孩子：寻找中序后继', '移除节点并维护链接'];
  return ['current = root', '比较 key 与 current.value', '若 key 更小，进入 left', '若 key 更大，进入 right', '空链接处创建新节点', '相等则保持树不变'];
}

function pseudoLine(event: TraceEvent | null) {
  if (!event) return -1;
  if (event.kind === 'compare') return 1;
  if (event.kind === 'descend') return event.direction === 'left' ? 2 : 3;
  if (event.kind === 'inserted') return 4;
  if (event.kind === 'duplicate') return 5;
  if (event.kind === 'successorSelected') return 4;
  if (event.kind === 'removed') return 5;
  if (event.kind === 'found') return 4;
  return 5;
}

function checkTree(tree: TreeView) {
  let count = 0;
  function visit(node: NodeView | null, min: number | null, max: number | null): boolean {
    if (!node) return true;
    count += 1;
    if ((min !== null && node.value <= min) || (max !== null && node.value >= max)) return false;
    return visit(node.left, min, node.value) && visit(node.right, node.value, max);
  }
  const ordered = visit(tree.root, null, null);
  return { ok: ordered && count === tree.len, count, message: !ordered ? '节点没有遵循左小右大的顺序。' : `节点数不一致：${count} / ${tree.len}` };
}

function treeHeight(root: NodeView | null): number {
  if (!root) return 0;
  return 1 + Math.max(treeHeight(root.left), treeHeight(root.right));
}

function nodeRelation(root: NodeView | null, id: string): string {
  function find(node: NodeView | null, parent: NodeView | null): string | null {
    if (!node) return null;
    if (node.id === id) return !parent ? '根节点' : parent.left?.id === id ? `节点 ${parent.value} 的左孩子` : `节点 ${parent.value} 的右孩子`;
    return find(node.left, node) ?? find(node.right, node);
  }
  return find(root, null) ?? '节点不在当前画面中';
}

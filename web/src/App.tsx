import { useEffect, useMemo, useReducer, useRef, useState } from 'react';
import type { StructureView, TraceView, TreeView, VisualizerCommand } from './types';
import TreeCanvas from './visualizer/TreeCanvas';
import { getVisualizerAdapter } from './visualizer/adapters';
import type { Operation, VisualizerSession } from './visualizer/session';
import { checkTree, getActiveIds, nodeRelation, treeHeight } from './visualizer/tree-model';
import { eventExplanation, eventTitle, pseudoLine, pseudoLines, shortEvent } from './visualizer/bst-lesson';
import { getAvailableLesson } from './visualizer/catalog';
import LessonHeader from './visualizer/LessonHeader';
import TreeMapLesson from './visualizer/TreeMapLesson';
import TrieLesson from './visualizer/TrieLesson';
import LinearLesson from './visualizer/LinearLesson';
import PriorityQueueLesson from './visualizer/PriorityQueueLesson';
import VectorLesson from './visualizer/VectorLesson';
import SinglyLinkedListLesson, { LinkedListLesson } from './visualizer/SinglyLinkedListLesson';
import HeapLesson from './visualizer/HeapLesson';
import AvlLesson from './visualizer/AvlLesson';
import RedBlackLesson from './visualizer/RedBlackLesson';
import BTreeLesson from './visualizer/BTreeLesson';
import BPlusLesson from './visualizer/BPlusLesson';
import HashLesson from './visualizer/HashLesson';

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
  | { type: 'loading' }
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

const defaultLessonId = 'binary-search-tree';
const emptyTree: TreeView = { kind: 'binaryTree', root: null, len: 0 };
function lessonIdFromLocation() {
  const match = window.location.hash.match(/^#lesson\/(.+)$/);
  return match && getAvailableLesson(match[1]) ? match[1] : defaultLessonId;
}
const initialState: State = {
  ready: false, error: null, operation: 'insert', input: '', tree: emptyTree,
  trace: null, cursor: 0, playing: false, speed: 760, selected: null,
};

function reducer(state: State, action: Action): State {
  switch (action.type) {
    case 'loading': return { ...state, ready: false, error: null, tree: emptyTree, trace: null, cursor: 0, playing: false, selected: null };
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

function BstLesson() {
  const [state, dispatch] = useReducer(reducer, initialState);
  const session = useRef<VisualizerSession<StructureView, TraceView> | null>(null);
  const lesson = getAvailableLesson(defaultLessonId)!;
  const busy = Boolean(state.trace && state.cursor < state.trace.events.length);
  const event = state.trace?.events[Math.min(state.cursor, Math.max(0, state.trace.events.length - 1))] ?? null;
  const tree = state.trace && (state.cursor >= state.trace.events.length || event?.kind === 'inserted')
    ? state.trace.after
    : state.trace?.before ?? state.tree;
  const activeIds = useMemo(() => getActiveIds(state.trace, state.cursor), [state.trace, state.cursor]);
  const invariant = useMemo(() => checkTree(tree), [tree]);

  useEffect(() => {
    let active = true;
    dispatch({ type: 'loading' });
    const adapter = lesson.adapterId ? getVisualizerAdapter<TreeView, TraceView>(lesson.adapterId) : null;
    const initialize = adapter?.initialize();
    if (!initialize) return;
    void initialize.then((visualizer) => {
      if (!active) return;
      session.current = visualizer;
      dispatch({ type: 'ready', tree: requireTreeView(visualizer.snapshot()) });
    }).catch((error: unknown) => {
      dispatch({ type: 'error', message: `Wasm 加载失败：${String(error)}` });
    });
    return () => { active = false; };
  }, [lesson.id, lesson.adapterId]);

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
    if (!state.input.trim() || !Number.isInteger(key) || key < -2147483648 || key > 2147483647) {
      dispatch({ type: 'error', message: '请输入 -2,147,483,648 到 2,147,483,647 之间的整数。' });
      return;
    }
    if (!session.current) return;
    try {
      const command: VisualizerCommand = {
        structureId: lesson.id,
        operation: state.operation,
        input: { kind: 'integerKey', value: key },
      };
      const trace = session.current.run(command);
      dispatch({ type: 'run', trace });
    } catch (error) {
      dispatch({ type: 'error', message: String(error) });
    }
  }

  function reset() {
    if (!session.current) return;
    dispatch({ type: 'reset', tree: requireTreeView(session.current.reset()) });
  }

  function loadExample() {
    if (!session.current) return;
    dispatch({ type: 'reset', tree: requireTreeView(session.current.loadExample()) });
  }

  function stepForward() { dispatch({ type: 'cursor', value: state.cursor + 1 }); }
  function stepBack() { dispatch({ type: 'cursor', value: state.cursor - 1 }); }

  return (
    <main className="app-shell">
      <LessonHeader selectedId={lesson.id} />

      <section className="intro" id="top">
        <div className="eyebrow"><span>DATA STRUCTURES</span><i />{lesson.category}</div>
        <h1>{lesson.label}，<em>每一步都看得见。</em></h1>
        <p>亲手执行插入、查找和删除。放慢算法，一步一步看清结构如何变化。</p>
      </section>

      <section className="workspace" aria-label="二叉搜索树交互区">
        <div className="canvas-card">
          <div className="card-heading">
            <div><span className="section-kicker">TREE CANVAS</span><h2>当前结构</h2></div>
            <div className="tree-count"><strong>{tree.len}</strong><span>个节点</span><i />高度 <strong>{treeHeight(tree.root)}</strong></div>
          </div>
          <div className="tree-canvas">
            <TreeCanvas tree={tree} activeIds={activeIds} currentEvent={state.trace && state.cursor < state.trace.events.length ? event : null} stepNumber={state.cursor + 1} stepCount={state.trace?.events.length ?? 0} selected={state.selected} onSelect={(id) => dispatch({ type: 'selected', value: id })} />
          </div>
          <div className="canvas-legend"><span><i className="legend-node" />节点</span><span><i className="legend-node active" />当前比较</span><span><i className="legend-edge" />搜索路径</span></div>
        </div>

        <aside className="side-column">
          <section className="panel operation-panel">
            <div className="panel-title"><span className="section-kicker">TRY IT YOURSELF</span><h2>执行操作</h2></div>
            <label className="field-label" htmlFor="operation">操作</label>
            <select id="operation" value={state.operation} onChange={(event) => dispatch({ type: 'operation', value: event.target.value as Operation })} disabled={!state.ready || busy}>
              {lesson.operations.map((operation) => <option key={operation.id} value={operation.id}>{operation.label}</option>)}
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

function requireTreeView(view: StructureView): TreeView {
  if (view.kind !== 'binaryTree') throw new Error(`The BST lesson received an incompatible view: ${view.kind}`);
  return view;
}

export default function App() {
  const [lessonId, setLessonId] = useState(lessonIdFromLocation);

  useEffect(() => {
    const syncLesson = () => setLessonId(lessonIdFromLocation());
    window.addEventListener('hashchange', syncLesson);
    if (window.location.hash !== `#lesson/${lessonId}`) {
      window.history.replaceState(null, '', `#lesson/${lessonId}`);
    }
    return () => window.removeEventListener('hashchange', syncLesson);
  }, [lessonId]);

  if (lessonId === 'vector') return <VectorLesson />;
  if (lessonId === 'stack') return <LinearLesson structureId="stack" />;
  if (lessonId === 'queue') return <LinearLesson key="queue" structureId="queue" />;
  if (lessonId === 'deque') return <LinearLesson key="deque" structureId="deque" />;
  if (lessonId === 'singly-linked-list') return <SinglyLinkedListLesson />;
  if (lessonId === 'doubly-linked-list') return <LinkedListLesson structureId="doubly-linked-list" mode="doubly" />;
  if (lessonId === 'binary-heap') return <HeapLesson />;
  if (lessonId === 'priority-queue') return <PriorityQueueLesson />;
  if (lessonId === 'avl-tree') return <AvlLesson />;
  if (lessonId === 'red-black-tree') return <RedBlackLesson />;
  if (lessonId === 'b-tree-map') return <BTreeLesson />;
  if (lessonId === 'b-tree-set') return <BTreeLesson key="b-tree-set" structureId="b-tree-set" />;
  if (lessonId === 'b-plus-tree-map') return <BPlusLesson />;
  if (lessonId === 'b-plus-tree-set') return <BPlusLesson key="b-plus-tree-set" structureId="b-plus-tree-set" />;
  if (lessonId === 'hash-map') return <HashLesson structureId="hash-map" />;
  if (lessonId === 'hash-set') return <HashLesson key="hash-set" structureId="hash-set" />;
  if (lessonId === 'binary-search-tree-map') return <TreeMapLesson structureId="binary-search-tree-map" />;
  if (lessonId === 'avl-tree-map') return <TreeMapLesson key="avl-tree-map" structureId="avl-tree-map" />;
  if (lessonId === 'red-black-tree-map') return <TreeMapLesson key="red-black-tree-map" structureId="red-black-tree-map" />;
  if (lessonId === 'trie') return <TrieLesson />;
  return <BstLesson />;
}

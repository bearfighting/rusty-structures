import LessonHeader from './LessonHeader';
import { useEffect, useMemo, useReducer, useRef } from 'react';
import type { LinkedListTraceEvent, LinkedListTraceView, LinkedListView, VisualizerCommand } from '../types';
import { getVisualizerAdapter } from './adapters';
import LinkedListCanvas from './LinkedListCanvas';
import type { VisualizerSession } from './session';
import type { SinglyLinkedListOperation } from './adapters/singlyLinkedList';
import type { DoublyLinkedListOperation } from './adapters/doublyLinkedList';

type LinkedListOperation = SinglyLinkedListOperation | DoublyLinkedListOperation;
type Props = { structureId: 'singly-linked-list' | 'doubly-linked-list'; mode: 'singly' | 'doubly' };

type State = {
  ready: boolean;
  error: string | null;
  operation: LinkedListOperation;
  value: string;
  list: LinkedListView;
  trace: LinkedListTraceView | null;
  cursor: number;
  playing: boolean;
  speed: number;
};
type Action =
  | { type: 'ready'; list: LinkedListView }
  | { type: 'error'; message: string }
  | { type: 'operation'; value: LinkedListOperation }
  | { type: 'value'; value: string }
  | { type: 'run'; trace: LinkedListTraceView }
  | { type: 'cursor'; value: number }
  | { type: 'playing'; value: boolean }
  | { type: 'speed'; value: number }
  | { type: 'reset'; list: LinkedListView };

const emptyList: LinkedListView = { kind: 'linkedList', linkMode: 'singly', nodes: [], headId: null, tailId: null, len: 0 };
const initialState: State = { ready: false, error: null, operation: 'pushBack', value: '', list: emptyList, trace: null, cursor: 0, playing: false, speed: 760 };

function reducer(state: State, action: Action): State {
  switch (action.type) {
    case 'ready': return { ...state, ready: true, list: action.list, error: null };
    case 'error': return { ...state, error: action.message };
    case 'operation': return { ...state, operation: action.value, error: null };
    case 'value': return { ...state, value: action.value };
    case 'run': return { ...state, trace: action.trace, cursor: 0, playing: false, error: null };
    case 'cursor': {
      const end = state.trace?.events.length ?? 0;
      const cursor = Math.max(0, Math.min(end, action.value));
      return { ...state, cursor, playing: cursor < end ? state.playing : false };
    }
    case 'playing': return { ...state, playing: action.value };
    case 'speed': return { ...state, speed: action.value };
    case 'reset': return { ...state, list: action.list, trace: null, cursor: 0, playing: false, error: null };
  }
}

export default function SinglyLinkedListLesson() {
  return <LinkedListLesson structureId="singly-linked-list" mode="singly" />;
}

export function LinkedListLesson({ structureId, mode }: Props) {
  const [state, dispatch] = useReducer(reducer, { ...initialState, list: { ...emptyList, linkMode: mode } });
  const session = useRef<VisualizerSession<LinkedListView, LinkedListTraceView> | null>(null);
  const busy = Boolean(state.trace && state.cursor < state.trace.events.length);
  const event = state.trace?.events[Math.min(state.cursor, Math.max(0, state.trace.events.length - 1))] ?? null;
  const list = state.trace && (state.cursor >= state.trace.events.length
    || event?.kind === 'nodeCreated'
    || event?.kind === 'linkCreated'
    || event?.kind === 'headChanged'
    || event?.kind === 'tailChanged') ? state.trace.after : state.trace?.before ?? state.list;
  const currentEvent = state.trace && state.cursor < state.trace.events.length ? event : null;
  const invariant = useMemo(() => checkLinkedList(list), [list]);

  useEffect(() => {
    let active = true;
    void getVisualizerAdapter<LinkedListView, LinkedListTraceView>(structureId).initialize().then((visualizer) => {
      if (!active) return;
      session.current = visualizer;
      dispatch({ type: 'ready', list: visualizer.snapshot() });
    }).catch((error: unknown) => dispatch({ type: 'error', message: `WASM 加载失败：${String(error)}` }));
    return () => { active = false; };
  }, [structureId]);

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
    if (!session.current) return;
    try {
      let input: VisualizerCommand['input'] = { kind: 'none' };
      if (state.operation === 'pushFront' || state.operation === 'pushBack') {
        const value = Number(state.value);
        if (!state.value.trim() || !Number.isInteger(value) || value < -2147483648 || value > 2147483647) {
          throw new Error('请输入 -2,147,483,648 到 2,147,483,647 之间的整数。');
        }
        input = { kind: 'value', value };
      }
      const command: VisualizerCommand = { structureId, operation: state.operation, input };
      dispatch({ type: 'run', trace: session.current.run(command) });
    } catch (error) {
      dispatch({ type: 'error', message: error instanceof Error ? error.message : String(error) });
    }
  }

  function reset() {
    if (session.current) dispatch({ type: 'reset', list: session.current.reset() });
  }

  function loadExample() {
    if (session.current) dispatch({ type: 'reset', list: session.current.loadExample() });
  }

  return <main className="app-shell">
    <LessonHeader selectedId={structureId} />
    <section className="intro" id="top">
      <div className="eyebrow"><span>DATA STRUCTURES</span><i />链表</div>
      
      <h1>{mode === 'singly' ? '单向链表' : '双向链表'}，<em>沿着链接走。</em></h1>
      <p>{mode === 'singly' ? '观察 HEAD、TAIL 与 next 指针如何改变。' : '观察 HEAD、TAIL、next 与 prev 如何成对维护。'}每个节点的地址身份独立于它保存的值。</p>
    </section>

    <section className="workspace" aria-label={`${mode === 'singly' ? '单向' : '双向'}链表交互区`}>
      <div className="canvas-card sequence-card">
          <div className="card-heading"><div><span className="section-kicker">LINKED NODES</span><h2>节点与{mode === 'singly' ? ' next 链接' : '双向链接'}</h2></div><div className="tree-count"><strong>{list.len}</strong><span>个节点</span><i />从左到右遍历</div></div>
        <div className="tree-canvas linked-list-canvas"><LinkedListCanvas list={list} currentEvent={currentEvent} step={state.cursor + 1} total={state.trace?.events.length ?? 0} /></div>
          <div className="canvas-legend"><span><i className="legend-node" />节点</span><span><i className="legend-node active" />当前节点</span><span><i className="legend-edge" />{mode === 'singly' ? 'next 链接' : 'next / prev 链接'}</span></div>
      </div>
      <aside className="side-column">
        <section className="panel operation-panel">
          <div className="panel-title"><span className="section-kicker">TRY IT YOURSELF</span><h2>执行操作</h2></div>
          <label className="field-label" htmlFor="list-operation">操作</label>
          <select id="list-operation" value={state.operation} onChange={(event) => dispatch({ type: 'operation', value: event.target.value as LinkedListOperation })} disabled={!state.ready || busy}>
            {linkedOperations(mode).map((operation) => <option value={operation.id} key={operation.id}>{operation.label}</option>)}
          </select>
          {(state.operation === 'pushFront' || state.operation === 'pushBack') ? <label className="field-label" htmlFor="list-value">整数值<input className="list-value-input" id="list-value" type="number" step="1" value={state.value} onChange={(event) => dispatch({ type: 'value', value: event.target.value })} onKeyDown={(event) => { if (event.key === 'Enter' && !busy) run(); }} placeholder="例如 21" disabled={!state.ready || busy} /></label> : <p className="operation-hint">{linkedOperationHint(state.operation)}</p>}
          <button className="primary-button list-run-button" onClick={run} disabled={!state.ready || busy}>{busy ? '逐步播放中' : '运行操作'}<span aria-hidden="true">↗</span></button>
          <div className="button-row"><button className="text-button" onClick={loadExample} disabled={!state.ready || busy}>载入示例</button><button className="text-button" onClick={reset} disabled={!state.ready}>清空</button></div>
          {state.error && <p className="error-message" role="alert">{state.error}</p>}
          <div className="divider" />
          <div className="playback-heading"><span className="section-kicker">PLAYBACK</span><span>{state.trace ? `${state.cursor} / ${state.trace.events.length}` : '等待操作'}</span></div>
          <div className="playback-controls">
            <button className="icon-button" onClick={() => dispatch({ type: 'cursor', value: state.cursor - 1 })} disabled={!state.trace || state.cursor === 0} aria-label="上一步">⟵</button>
            <button className="play-button" onClick={() => dispatch({ type: 'playing', value: !state.playing })} disabled={!state.trace || (!state.playing && state.cursor >= state.trace.events.length)} aria-label={state.playing ? '暂停' : '播放'}>{state.playing ? 'Ⅱ' : '▶'}</button>
            <button className="icon-button" onClick={() => dispatch({ type: 'cursor', value: state.cursor + 1 })} disabled={!state.trace || state.cursor >= state.trace.events.length} aria-label="下一步">⟶</button>
            <button className="icon-button reset-icon" onClick={() => dispatch({ type: 'cursor', value: 0 })} disabled={!state.trace || state.cursor === 0} aria-label="回到开始">↺</button>
            <label className="speed-control"><span>速度</span><input aria-label="播放速度" type="range" min="250" max="1500" step="50" value={1750 - state.speed} onChange={(event) => dispatch({ type: 'speed', value: 1750 - Number(event.target.value) })} /><span>{state.speed < 500 ? '快' : state.speed > 1050 ? '慢' : '中'}</span></label>
          </div>
        </section>
        <section className="panel lesson-panel" aria-live="polite">
          <div className="panel-title compact"><span className="section-kicker">LINK STEP</span><span className="step-number">{state.trace ? `${Math.min(state.cursor + 1, state.trace.events.length)} / ${state.trace.events.length}` : '—'}</span></div>
          <h2>{listEventTitle(event, state.trace, state.cursor)}</h2>
          <p className="step-explanation">{listEventExplanation(event, state.trace, state.cursor, mode)}</p>
          <div className="pseudocode-title">伪代码 <span>{mode === 'singly' ? 'SINGLY' : 'DOUBLY'} LINKED LIST</span></div>
          <ol className="pseudocode">{listPseudocode(state.operation).map((line, index) => <li key={line} className={index === listPseudoLine(event) ? 'current' : ''}><code>{line}</code></li>)}</ol>
          <div className={`invariant ${invariant ? 'ok' : 'bad'}`}><span className="invariant-icon">{invariant ? '✓' : '!'}</span><div><strong>{invariant ? '链表链接结构一致' : '检测到链接错误'}</strong><span>{invariant ? `从 HEAD 可达 ${list.len} 个唯一节点，末尾 next 为空。` : '节点数量、HEAD/TAIL 或 next 链接与快照不一致。'}</span></div></div>
        </section>
      </aside>
    </section>

    <section className="timeline-card" aria-label="链表操作步骤时间线">
      <div className="timeline-heading"><div><span className="section-kicker">OPERATION TRACE</span><h2>算法步骤</h2></div><span className="timeline-hint">选择步骤可跳转</span></div>
      {state.trace?.events.length ? <ol className="event-list">{state.trace.events.map((step, index) => <li key={`${index}-${step.kind}`}><button className={index === Math.min(state.cursor, state.trace!.events.length - 1) ? 'selected' : ''} onClick={() => dispatch({ type: 'cursor', value: index })}><span className="event-index">{String(index + 1).padStart(2, '0')}</span><span>{listShortEvent(step)}</span><i>{index < state.cursor ? '✓' : '·'}</i></button></li>)}</ol> : <div className="timeline-empty">执行操作后，这里会显示节点创建、链接变化和 HEAD/TAIL 更新。</div>}
    </section>
    <footer><span>由 Rust {mode === 'singly' ? 'SinglyLinkedList' : 'DoublyLinkedList'} 驱动</span><span>Tree Lab · 原型 0.3</span><span>学习算法，从观察开始。</span></footer>
  </main>;
}

function checkLinkedList(list: LinkedListView): boolean {
  if (list.nodes.length !== list.len) return false;
  if ((list.nodes[0]?.id ?? null) !== list.headId || (list.nodes.at(-1)?.id ?? null) !== list.tailId) return false;
  if (new Set(list.nodes.map((node) => node.id)).size !== list.len) return false;
  return list.nodes.every((node, index) => node.nextId === (list.nodes[index + 1]?.id ?? null)
    && node.prevId === (list.linkMode === 'doubly' ? (list.nodes[index - 1]?.id ?? null) : null));
}

function linkedOperations(mode: Props['mode']) {
  const common = [
    { id: 'pushFront', label: '从头部添加' },
    { id: 'pushBack', label: '从尾部添加' },
    { id: 'popFront', label: '移除头部' },
  ];
  if (mode === 'doubly') common.push({ id: 'popBack', label: '移除尾部' });
  common.push({ id: 'front', label: '查看头部' }, { id: 'back', label: '查看尾部' });
  return common;
}

function linkedOperationHint(operation: LinkedListOperation) {
  if (operation === 'popFront') return '移除并返回当前 HEAD 节点；空链表会报告空操作。';
  if (operation === 'popBack') return '移除并返回当前 TAIL 节点；prev 链接让操作保持 O(1)。';
  if (operation === 'front') return '读取 HEAD 节点的值，不改变链表。';
  return '读取维护的 TAIL 节点值，不遍历整个链表。';
}

function listShortEvent(event: LinkedListTraceEvent): string {
  switch (event.kind) {
    case 'nodeCreated': return `创建节点 ${event.value}`;
    case 'linkCreated': return `连接 ${event.direction} 指针`;
    case 'linkRemoved': return `断开 ${event.direction} 链接`;
    case 'headChanged': return '更新 HEAD';
    case 'tailChanged': return '更新 TAIL';
    case 'nodeRemoved': return `移除节点 ${event.value}`;
    case 'nodeVisited': return `访问${event.endpoint === 'head' ? '头部' : '尾部'} ${event.value}`;
    case 'empty': return '链表为空';
  }
}

function listEventTitle(event: LinkedListTraceEvent | null, trace: LinkedListTraceView | null, cursor: number) {
  if (!trace) return '准备开始';
  if (cursor >= trace.events.length) return '操作已完成';
  return event ? listShortEvent(event) : '操作已完成';
}

function listEventExplanation(event: LinkedListTraceEvent | null, trace: LinkedListTraceView | null, cursor: number, mode: Props['mode']) {
  if (!trace) return `选择从头部或尾部添加节点${mode === 'doubly' ? '，也可以从两端移除' : '，也可以查看或移除 HEAD'}。结构变化来自 Rust 的链表实现。`;
  if (cursor >= trace.events.length) return `操作完成。链表现在包含 ${trace.after.len} 个节点。`;
  if (!event) return '观察当前步骤。';
  switch (event.kind) {
    case 'nodeCreated': return `新节点 ${event.value} 已获得独立身份，接下来更新链表链接。`;
    case 'linkCreated': return event.direction === 'next' ? 'next 链接指向后继节点。' : 'prev 链接指向前驱节点，帮助从尾部向前访问。';
    case 'linkRemoved': return `${event.direction} 链接已断开，避免保留指向已移除节点的关系。`;
    case 'headChanged': return event.toId ? 'HEAD 现在指向新的首节点。' : '链表已空，HEAD 设为 None。';
    case 'tailChanged': return event.toId ? '维护的 TAIL 指针现在指向最后一个节点。' : '最后一个节点已移除，TAIL 设为 None。';
    case 'nodeRemoved': return `节点 ${event.value} 已从头部断开；其余节点顺序不变。`;
    case 'nodeVisited': return `${event.endpoint === 'head' ? 'HEAD' : 'TAIL'} 直接提供值 ${event.value}；这两个端点读取均为 O(1)。`;
    case 'empty': return '链表为空，没有节点可读取或移除。';
  }
}

function listPseudocode(operation: LinkedListOperation) {
  if (operation === 'pushFront') return ['创建新节点', '新节点 next 指向当前 HEAD', 'HEAD 更新为新节点', '若链表原来为空，TAIL 也指向它'];
  if (operation === 'pushBack') return ['创建新节点，next 为空', '旧 TAIL 的 next 指向新节点', 'TAIL 更新为新节点'];
  if (operation === 'popFront') return ['读取当前 HEAD', 'HEAD 更新为首节点的 next', '断开并返回旧 HEAD', '若链表为空，清空 TAIL'];
  if (operation === 'popBack') return ['读取当前 TAIL', '通过 prev 找到新 TAIL', '断开旧 TAIL 的双向链接', '返回旧 TAIL'];
  if (operation === 'front') return ['读取 HEAD 节点值', '不改变链接'];
  return ['读取维护的 TAIL 节点值', '不遍历链表', '不改变链接'];
}

function listPseudoLine(event: LinkedListTraceEvent | null) {
  if (!event) return -1;
  if (event.kind === 'nodeCreated') return 0;
  if (event.kind === 'linkCreated') return 1;
  if (event.kind === 'linkRemoved') return 1;
  if (event.kind === 'headChanged') return 2;
  if (event.kind === 'tailChanged') return 3;
  if (event.kind === 'nodeRemoved') return 2;
  if (event.kind === 'nodeVisited') return 0;
  return 0;
}

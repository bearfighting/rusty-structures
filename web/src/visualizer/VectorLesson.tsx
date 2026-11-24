import LessonHeader from './LessonHeader';
import { useEffect, useMemo, useReducer, useRef } from 'react';
import type { Dispatch } from 'react';
import type { SequenceTraceEvent, SequenceTraceView, SequenceView, VisualizerCommand } from '../types';
import { getVisualizerAdapter } from './adapters';
import SequenceCanvas from './SequenceCanvas';
import type { VisualizerSession } from './session';
import type { VectorOperation } from './adapters/vector';

type State = {
  ready: boolean;
  error: string | null;
  operation: VectorOperation;
  value: string;
  index: string;
  additional: string;
  sequence: SequenceView;
  trace: SequenceTraceView | null;
  cursor: number;
  playing: boolean;
  speed: number;
};
type Action =
  | { type: 'ready'; sequence: SequenceView }
  | { type: 'error'; message: string }
  | { type: 'operation'; value: VectorOperation }
  | { type: 'value'; value: string }
  | { type: 'index'; value: string }
  | { type: 'additional'; value: string }
  | { type: 'run'; trace: SequenceTraceView }
  | { type: 'cursor'; value: number }
  | { type: 'playing'; value: boolean }
  | { type: 'speed'; value: number }
  | { type: 'reset'; sequence: SequenceView };

const emptySequence: SequenceView = { kind: 'sequence', items: [], len: 0, capacity: 0 };
const initialState: State = {
  ready: false, error: null, operation: 'push', value: '', index: '', additional: '',
  sequence: emptySequence, trace: null, cursor: 0, playing: false, speed: 760,
};

function reducer(state: State, action: Action): State {
  switch (action.type) {
    case 'ready': return { ...state, ready: true, sequence: action.sequence, error: null };
    case 'error': return { ...state, error: action.message };
    case 'operation': return { ...state, operation: action.value, error: null };
    case 'value': return { ...state, value: action.value };
    case 'index': return { ...state, index: action.value };
    case 'additional': return { ...state, additional: action.value };
    case 'run': return { ...state, trace: action.trace, cursor: 0, playing: false, error: null };
    case 'cursor': {
      const end = state.trace?.events.length ?? 0;
      const cursor = Math.max(0, Math.min(end, action.value));
      return { ...state, cursor, playing: cursor < end ? state.playing : false };
    }
    case 'playing': return { ...state, playing: action.value };
    case 'speed': return { ...state, speed: action.value };
    case 'reset': return { ...state, sequence: action.sequence, trace: null, cursor: 0, playing: false, error: null };
  }
}

export default function VectorLesson() {
  const [state, dispatch] = useReducer(reducer, initialState);
  const session = useRef<VisualizerSession<SequenceView, SequenceTraceView> | null>(null);
  const busy = Boolean(state.trace && state.cursor < state.trace.events.length);
  const event = state.trace?.events[Math.min(state.cursor, Math.max(0, state.trace.events.length - 1))] ?? null;
  const sequence = state.trace && event?.kind === 'capacityChanged' && state.operation !== 'reserve'
    ? { ...state.trace.before, capacity: state.trace.after.capacity }
    : state.trace && (
    state.cursor >= state.trace.events.length
    || event?.kind === 'inserted'
    || (state.operation === 'reserve' && event?.kind === 'capacityChanged')
  ) ? state.trace.after : state.trace?.before ?? state.sequence;
  const invariant = useMemo(() => sequence.items.length === sequence.len
    && new Set(sequence.items.map((item) => item.id)).size === sequence.len, [sequence]);

  useEffect(() => {
    let active = true;
    void getVisualizerAdapter<SequenceView, SequenceTraceView, VectorOperation>('vector').initialize().then((visualizer) => {
      if (!active) return;
      session.current = visualizer;
      dispatch({ type: 'ready', sequence: visualizer.snapshot() });
    }).catch((error: unknown) => dispatch({ type: 'error', message: `WASM 加载失败：${String(error)}` }));
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
    if (!session.current) return;
    try {
      const input = commandInput(state);
      const command: VisualizerCommand = { structureId: 'vector', operation: state.operation, input };
      dispatch({ type: 'run', trace: session.current.run(command) });
    } catch (error) {
      dispatch({ type: 'error', message: error instanceof Error ? error.message : String(error) });
    }
  }

  function reset() {
    if (session.current) dispatch({ type: 'reset', sequence: session.current.reset() });
  }

  function loadExample() {
    if (session.current) dispatch({ type: 'reset', sequence: session.current.loadExample() });
  }

  const currentEvent = state.trace && state.cursor < state.trace.events.length ? event : null;
  const steps = vectorPseudocode(state.operation);

  return <main className="app-shell">
      <LessonHeader selectedId="vector" />

    <section className="intro" id="top">
      <div className="eyebrow"><span>DATA STRUCTURES</span><i />线性容器</div>
      
      <h1>动态数组，<em>每个索引都看得见。</em></h1>
      <p>观察 Rust Vector 如何追加、插入、移动和读取元素，以及容量何时增长。</p>
    </section>

    <section className="workspace" aria-label="Vector 交互区">
      <div className="canvas-card sequence-card">
        <div className="card-heading"><div><span className="section-kicker">SEQUENCE VIEW</span><h2>连续存储布局</h2></div><div className="tree-count"><strong>{sequence.len}</strong><span>个元素</span><i />容量 <strong>{sequence.capacity}</strong></div></div>
        <div className="tree-canvas sequence-canvas"><SequenceCanvas sequence={sequence} currentEvent={currentEvent} /></div>
        <div className="canvas-legend"><span><i className="legend-node" />元素</span><span><i className="legend-node active" />当前访问/移动</span><span><i className="legend-edge" />索引位置</span></div>
      </div>

      <aside className="side-column">
        <section className="panel operation-panel">
          <div className="panel-title"><span className="section-kicker">TRY IT YOURSELF</span><h2>执行操作</h2></div>
          <label className="field-label" htmlFor="vector-operation">操作</label>
          <select id="vector-operation" value={state.operation} onChange={(event) => dispatch({ type: 'operation', value: event.target.value as VectorOperation })} disabled={!state.ready || busy}>
            <option value="push">追加到末尾</option><option value="pop">移除末尾</option><option value="insert">按索引插入</option><option value="remove">按索引移除</option><option value="get">读取索引</option><option value="reserve">预留额外容量</option>
          </select>
          <VectorInputs state={state} dispatch={dispatch} disabled={!state.ready || busy} onRun={run} />
          <div className="vector-action-row"><button className="primary-button" onClick={run} disabled={!state.ready || busy}>{busy ? '逐步播放中' : '运行操作'}<span aria-hidden="true">↗</span></button><div className="button-row"><button className="text-button" onClick={loadExample} disabled={!state.ready || busy}>载入示例</button><button className="text-button" onClick={reset} disabled={!state.ready}>清空</button></div></div>
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
          <div className="panel-title compact"><span className="section-kicker">VECTOR STEP</span><span className="step-number">{state.trace ? `${Math.min(state.cursor + 1, state.trace.events.length)} / ${state.trace.events.length}` : '—'}</span></div>
          <h2>{vectorEventTitle(event, state.trace, state.cursor)}</h2>
          <p className="step-explanation">{vectorEventExplanation(event, state.trace, state.cursor, sequence)}</p>
          <div className="pseudocode-title">伪代码 <span>VECTOR OPERATIONS</span></div>
          <ol className="pseudocode">{steps.map((line, index) => <li key={line} className={index === vectorPseudoLine(event) ? 'current' : ''}><code>{line}</code></li>)}</ol>
          <div className={`invariant ${invariant ? 'ok' : 'bad'}`}><span className="invariant-icon">{invariant ? '✓' : '!'}</span><div><strong>{invariant ? '连续存储状态一致' : '检测到状态错误'}</strong><span>{invariant ? `快照包含 ${sequence.len} 个元素，ID 唯一。` : '元素数量或视觉 ID 与快照不一致。'}</span></div></div>
        </section>
      </aside>
    </section>

    <section className="timeline-card" aria-label="Vector 操作步骤时间线">
      <div className="timeline-heading"><div><span className="section-kicker">OPERATION TRACE</span><h2>算法步骤</h2></div><span className="timeline-hint">选择步骤可跳转</span></div>
      {state.trace?.events.length ? <ol className="event-list">{state.trace.events.map((step, index) => <li key={`${index}-${step.kind}`}><button className={index === Math.min(state.cursor, state.trace!.events.length - 1) ? 'selected' : ''} onClick={() => dispatch({ type: 'cursor', value: index })}><span className="event-index">{String(index + 1).padStart(2, '0')}</span><span>{vectorShortEvent(step)}</span><i>{index < state.cursor ? '✓' : '·'}</i></button></li>)}</ol> : <div className="timeline-empty">执行一次操作，这里会显示 Vector 的索引访问、移位和容量变化。</div>}
    </section>
    <footer><span>由 Rust Vector 驱动</span><span>Tree Lab · 原型 0.2</span><span>学习算法，从观察开始。</span></footer>
  </main>;
}

function VectorInputs({ state, dispatch, disabled, onRun }: { state: State; dispatch: Dispatch<Action>; disabled: boolean; onRun: () => void }) {
  const numericInput = (id: string, label: string, value: string, onChange: (value: string) => void, placeholder: string) => <label className="vector-input-field" htmlFor={id}>{label}<input id={id} type="number" step="1" value={value} onChange={(event) => onChange(event.target.value)} onKeyDown={(event) => { if (event.key === 'Enter' && !disabled) onRun(); }} placeholder={placeholder} disabled={disabled} /></label>;
  if (state.operation === 'pop') return <p className="operation-hint">移除并返回最后一个元素；空 Vector 会报告空操作。</p>;
  if (state.operation === 'push') return numericInput('vector-value', '整数值', state.value, (value) => dispatch({ type: 'value', value }), '例如 42');
  if (state.operation === 'insert') return <div className="vector-input-pair">{numericInput('vector-index', '插入索引', state.index, (value) => dispatch({ type: 'index', value }), '0 到长度')}{numericInput('vector-value', '整数值', state.value, (value) => dispatch({ type: 'value', value }), '例如 42')}</div>;
  if (state.operation === 'reserve') return numericInput('vector-capacity', '额外容量', state.additional, (value) => dispatch({ type: 'additional', value }), '例如 8');
  return numericInput('vector-index', '索引', state.index, (value) => dispatch({ type: 'index', value }), '例如 2');
}

function commandInput(state: State): VisualizerCommand['input'] {
  if (state.operation === 'pop') return { kind: 'none' };
  if (state.operation === 'push') return { kind: 'value', value: requireI32(state.value) };
  if (state.operation === 'insert') return { kind: 'indexedValue', index: requireIndex(state.index), value: requireI32(state.value) };
  if (state.operation === 'reserve') return { kind: 'capacity', additional: requireIndex(state.additional) };
  return { kind: 'index', index: requireIndex(state.index) };
}

function requireI32(value: string): number {
  const number = Number(value);
  if (!value.trim() || !Number.isInteger(number) || number < -2147483648 || number > 2147483647) throw new Error('请输入 -2,147,483,648 到 2,147,483,647 之间的整数。');
  return number;
}

function requireIndex(value: string): number {
  const number = Number(value);
  if (!value.trim() || !Number.isInteger(number) || number < 0 || number > 64) throw new Error('索引或额外容量必须是 0 到 64 之间的整数。');
  return number;
}

function vectorEventTitle(event: SequenceTraceEvent | null, trace: SequenceTraceView | null, cursor: number) {
  if (!trace) return '准备开始';
  if (cursor >= trace.events.length) return '操作已完成';
  if (!event) return '操作已完成';
  return vectorShortEvent(event);
}

function vectorShortEvent(event: SequenceTraceEvent) {
  switch (event.kind) {
    case 'capacityChanged': return `容量扩展：${event.oldCapacity} → ${event.newCapacity}`;
    case 'shift': return `值 ${event.value}：索引 ${event.fromIndex} → ${event.toIndex}`;
    case 'inserted': return `插入 ${event.value} 到索引 ${event.index}`;
    case 'removed': return `移除索引 ${event.index} 的 ${event.value}`;
    case 'accessed': return `读取索引 ${event.index}：${event.value}`;
    case 'missingIndex': return `索引 ${event.index} 越界`;
    case 'empty': return 'Vector 为空';
    case 'reserveUnchanged': return `容量不变：${event.capacity}`;
  }
}

function vectorEventExplanation(event: SequenceTraceEvent | null, trace: SequenceTraceView | null, cursor: number, sequence: SequenceView) {
  if (!trace) return '选择一项操作。所有结构变化和容量数据都来自 Rust 的 Vector 实现。';
  if (cursor >= trace.events.length) return `操作完成，Vector 当前有 ${sequence.len} 个元素，容量为 ${sequence.capacity}。`;
  if (!event) return '观察当前步骤。';
  switch (event.kind) {
    case 'capacityChanged': return `Vector 为后续元素申请了更大的连续存储空间：容量从 ${event.oldCapacity} 变为 ${event.newCapacity}。`;
    case 'shift': return `为了保持连续索引，将值 ${event.value} 从槽位 ${event.fromIndex} 移到槽位 ${event.toIndex}。`;
    case 'inserted': return `值 ${event.value} 已写入索引 ${event.index}，长度增加一。`;
    case 'removed': return `索引 ${event.index} 的值 ${event.value} 被移除；Vector 保持其余元素的相对顺序。`;
    case 'accessed': return `索引 ${event.index} 有效，读取到 ${event.value}。索引读取不改变 Vector。`;
    case 'missingIndex': return `索引 ${event.index} 超出长度 ${event.len}，操作不会改变 Vector。`;
    case 'empty': return 'Vector 为空，没有元素可弹出。';
    case 'reserveUnchanged': return `现有容量 ${event.capacity} 已满足请求，不需要重新分配。`;
  }
}

function vectorPseudocode(operation: VectorOperation) {
  if (operation === 'push') return ['确认还有可用容量', '必要时申请更大的连续空间', '在 len 位置写入新值', 'len 增加一'];
  if (operation === 'pop') return ['检查 Vector 是否为空', '读取最后一个元素', 'len 减少一并返回该值'];
  if (operation === 'insert') return ['验证 index ≤ len', '必要时申请更大容量', '将 index 及之后的元素右移', '在 index 写入新值', 'len 增加一'];
  if (operation === 'remove') return ['检查 index < len', '读取并移除目标元素', '将后续元素左移', 'len 减少一'];
  if (operation === 'get') return ['检查 index < len', '返回对应元素的引用', '越界时返回 None'];
  return ['计算 len + additional', '容量足够时不做改动', '否则申请更大的连续空间'];
}

function vectorPseudoLine(event: SequenceTraceEvent | null) {
  if (!event) return -1;
  if (event.kind === 'capacityChanged' || event.kind === 'reserveUnchanged') return 1;
  if (event.kind === 'shift') return 2;
  if (event.kind === 'inserted') return 3;
  if (event.kind === 'removed') return 1;
  if (event.kind === 'accessed' || event.kind === 'missingIndex') return 0;
  return 0;
}

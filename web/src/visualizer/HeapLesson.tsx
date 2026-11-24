import LessonHeader from './LessonHeader';
import { useEffect, useReducer, useRef } from 'react';
import type { HeapTraceEvent, HeapTraceView, HeapView, VisualizerCommand } from '../types';
import type { VisualizerSession } from './session';
import { getVisualizerAdapter } from './adapters';
import type { HeapOperation } from './adapters/heap';
import HeapCanvas from './HeapCanvas';

type State = { ready: boolean; error: string | null; operation: HeapOperation; value: string; heap: HeapView; trace: HeapTraceView | null; cursor: number; playing: boolean; speed: number };
type Action = { type: 'ready'; heap: HeapView } | { type: 'error'; message: string } | { type: 'operation'; value: HeapOperation } | { type: 'value'; value: string } | { type: 'run'; trace: HeapTraceView } | { type: 'cursor'; value: number } | { type: 'playing'; value: boolean } | { type: 'speed'; value: number } | { type: 'reset'; heap: HeapView };
const emptyHeap: HeapView = { kind: 'heap', items: [], len: 0 };
const initial: State = { ready: false, error: null, operation: 'push', value: '', heap: emptyHeap, trace: null, cursor: 0, playing: false, speed: 760 };
function reducer(state: State, action: Action): State {
  switch (action.type) {
    case 'ready': return { ...state, ready: true, heap: action.heap };
    case 'error': return { ...state, error: action.message };
    case 'operation': return { ...state, operation: action.value, error: null };
    case 'value': return { ...state, value: action.value };
    case 'run': return { ...state, trace: action.trace, cursor: 0, playing: false, error: null };
    case 'cursor': { const cursor = Math.max(0, Math.min(state.trace?.events.length ?? 0, action.value)); return { ...state, cursor, playing: cursor < (state.trace?.events.length ?? 0) ? state.playing : false }; }
    case 'playing': return { ...state, playing: action.value };
    case 'speed': return { ...state, speed: action.value };
    case 'reset': return { ...state, heap: action.heap, trace: null, cursor: 0, playing: false, error: null };
  }
}

export default function HeapLesson() {
  const [state, dispatch] = useReducer(reducer, initial);
  const session = useRef<VisualizerSession<HeapView, HeapTraceView, HeapOperation> | null>(null);
  const busy = Boolean(state.trace && state.cursor < state.trace.events.length);
  const event = state.trace?.events[Math.min(state.cursor, Math.max(0, state.trace.events.length - 1))] ?? null;
  const heap = state.trace ? project(state.trace, state.cursor) : state.heap;
  useEffect(() => { let active = true; void getVisualizerAdapter<HeapView, HeapTraceView, HeapOperation>('binary-heap').initialize().then((adapter) => { if (active) { session.current = adapter; dispatch({ type: 'ready', heap: adapter.snapshot() }); } }).catch((error: unknown) => dispatch({ type: 'error', message: `WASM 加载失败：${String(error)}` })); return () => { active = false; }; }, []);
  useEffect(() => { if (!state.playing) return; let frame = 0; let previous = 0; let elapsed = 0; const tick = (now: number) => { if (previous) elapsed += now - previous; previous = now; if (elapsed >= state.speed) { elapsed = 0; dispatch({ type: 'cursor', value: state.cursor + 1 }); } frame = requestAnimationFrame(tick); }; frame = requestAnimationFrame(tick); return () => cancelAnimationFrame(frame); }, [state.playing, state.cursor, state.speed]);
  function run() { if (!session.current) return; try { let input: VisualizerCommand['input'] = { kind: 'none' }; if (state.operation === 'push') { const value = Number(state.value); if (!state.value.trim() || !Number.isInteger(value) || value < -2147483648 || value > 2147483647) throw new Error('请输入有效的 32 位整数。'); input = { kind: 'value', value }; } dispatch({ type: 'run', trace: session.current.run({ structureId: 'binary-heap', operation: state.operation, input }) }); } catch (error) { dispatch({ type: 'error', message: error instanceof Error ? error.message : String(error) }); } }
  function reset(loadExample = false) { if (session.current) dispatch({ type: 'reset', heap: loadExample ? session.current.loadExample() : session.current.reset() }); }
  const currentEvent = state.trace && state.cursor < state.trace.events.length ? event : null;
  const completed = state.trace && state.cursor >= state.trace.events.length;
  const heading = !state.trace ? '准备开始' : completed ? '操作已完成' : event ? heapEventLabel(event) : '操作已完成';
  const explanation = !state.trace ? '最大堆让最大值始终位于根节点。逐步观察插入上浮、删除下沉中的比较和交换。' : completed ? `操作完成，堆中有 ${heap.len} 个元素。` : event ? heapEventExplanation(event) : '';
  return <main className="app-shell">
    <LessonHeader selectedId="binary-heap" />
    <section className="intro" id="top"><div className="eyebrow"><span>DATA STRUCTURES</span><i />优先结构</div><h1>二叉堆，<em>最大值始终在根。</em></h1><p>同步观察数组布局与树形关系，跟随 Rust 的比较和交换逐步恢复最大堆不变量。</p></section>
    <section className="workspace" aria-label="二叉堆交互区"><div className="canvas-card sequence-card"><div className="card-heading"><div><span className="section-kicker">HEAP CANVAS</span><h2>最大堆结构</h2></div><div className="tree-count"><strong>{heap.len}</strong><span>个节点</span><i />层数 <strong>{heap.len ? Math.floor(Math.log2(heap.len)) + 1 : 0}</strong></div></div><div className="tree-canvas heap-canvas"><HeapCanvas heap={heap} event={currentEvent} /></div><div className="canvas-legend"><span><i className="legend-node" />节点与数组槽位</span><span><i className="legend-node active" />当前比较/交换</span><span><i className="legend-edge" />父子关系</span></div></div>
      <aside className="side-column"><section className="panel operation-panel"><div className="panel-title"><span className="section-kicker">TRY IT YOURSELF</span><h2>执行操作</h2></div><label className="field-label" htmlFor="heap-operation">操作</label><select id="heap-operation" value={state.operation} onChange={(e) => dispatch({ type: 'operation', value: e.target.value as HeapOperation })} disabled={!state.ready || busy}><option value="push">插入元素</option><option value="pop">移除最大值</option><option value="peek">查看最大值</option></select>{state.operation === 'push' ? <><label className="field-label" htmlFor="heap-value">整数值</label><div className="input-row"><input id="heap-value" type="number" step="1" value={state.value} placeholder="例如 22" onChange={(e) => dispatch({ type: 'value', value: e.target.value })} onKeyDown={(e) => { if (e.key === 'Enter' && !busy) run(); }} disabled={!state.ready || busy} /><button className="primary-button" onClick={run} disabled={!state.ready || busy}>运行操作 ↗</button></div></> : <><p className="operation-hint">{state.operation === 'pop' ? '移除并返回堆中的最大值；空堆不会改变。' : '读取根节点的最大值，不改变堆。'}</p><button className="primary-button heap-run" onClick={run} disabled={!state.ready || busy}>运行操作 ↗</button></>}<div className="button-row"><button className="text-button" onClick={() => reset(true)} disabled={!state.ready || busy}>载入示例</button><button className="text-button" onClick={() => reset()} disabled={!state.ready}>清空</button></div>{state.error && <p className="error-message" role="alert">{state.error}</p>}<div className="divider"/><div className="playback-heading"><span className="section-kicker">PLAYBACK</span><span>{state.trace ? `${state.cursor} / ${state.trace.events.length}` : '等待操作'}</span></div><div className="playback-controls"><button className="icon-button" onClick={() => dispatch({ type: 'cursor', value: state.cursor - 1 })} disabled={!state.trace || !state.cursor} aria-label="上一步">⟵</button><button className="play-button" onClick={() => dispatch({ type: 'playing', value: !state.playing })} disabled={!state.trace || (!state.playing && state.cursor >= state.trace.events.length)} aria-label={state.playing ? '暂停' : '播放'}>{state.playing ? 'Ⅱ' : '▶'}</button><button className="icon-button" onClick={() => dispatch({ type: 'cursor', value: state.cursor + 1 })} disabled={!state.trace || state.cursor >= state.trace.events.length} aria-label="下一步">⟶</button><button className="icon-button reset-icon" onClick={() => dispatch({ type: 'cursor', value: 0 })} disabled={!state.trace || !state.cursor} aria-label="回到开始">↺</button><label className="speed-control"><span>速度</span><input aria-label="播放速度" type="range" min="250" max="1500" step="50" value={1750 - state.speed} onChange={(e) => dispatch({ type: 'speed', value: 1750 - Number(e.target.value) })}/><span>中</span></label></div></section>
        <section className="panel lesson-panel" aria-live="polite"><div className="panel-title compact"><span className="section-kicker">HEAP STEP</span><span className="step-number">{state.trace ? `${Math.min(state.cursor + 1, state.trace.events.length)} / ${state.trace.events.length}` : '—'}</span></div><h2>{heading}</h2><p className="step-explanation">{explanation}</p><div className="pseudocode-title">伪代码 <span>BINARY HEAP</span></div><ol className="pseudocode">{heapPseudo(state.operation).map((line, i) => <li className={event ? i === heapPseudoLine(event) ? 'current' : '' : ''} key={line}><code>{line}</code></li>)}</ol><div className={`invariant ${checkHeap(heap) ? 'ok' : 'bad'}`}><span className="invariant-icon">{checkHeap(heap) ? '✓' : '!'}</span><div><strong>{checkHeap(heap) ? '最大堆不变量成立' : '检测到堆序错误'}</strong><span>{checkHeap(heap) ? `已检查 ${heap.len} 个节点及其父子关系。` : '父节点必须不小于子节点。'}</span></div></div></section></aside></section>
    <section className="timeline-card" aria-label="堆操作时间线"><div className="timeline-heading"><div><span className="section-kicker">OPERATION TRACE</span><h2>算法步骤</h2></div><span className="timeline-hint">选择步骤可跳转</span></div>{state.trace?.events.length ? <ol className="event-list">{state.trace.events.map((step, i) => <li key={`${i}-${step.kind}`}><button className={i === Math.min(state.cursor, state.trace!.events.length - 1) ? 'selected' : ''} onClick={() => dispatch({ type: 'cursor', value: i })}><span className="event-index">{String(i + 1).padStart(2, '0')}</span><span>{heapEventLabel(step)}</span><i>{i < state.cursor ? '✓' : '·'}</i></button></li>)}</ol> : <div className="timeline-empty">执行操作后，这里会显示 Rust BinaryHeap 实际经过的比较、交换和根节点变化。</div>}</section><footer><span>由 Rust BinaryHeap 驱动</span><span>Tree Lab · 原型 0.3</span><span>学习算法，从观察开始。</span></footer>
  </main>;
}

function project(trace: HeapTraceView, cursor: number): HeapView {
  const items = trace.before.items.map((item) => ({ ...item }));
  for (const event of trace.events.slice(0, cursor)) {
    if (event.kind === 'inserted') items.push({ id: event.itemId, value: event.value, index: event.index });
    if (event.kind === 'rootRemoved') {
      items.pop();
      items.shift();
      if (event.replacementId) items.unshift({ id: event.replacementId, value: event.replacementValue!, index: 0 });
    }
    if (event.kind === 'swapped') [items[event.firstIndex], items[event.secondIndex]] = [items[event.secondIndex], items[event.firstIndex]];
  }
  items.forEach((item, index) => { item.index = index; });
  return { kind: 'heap', items, len: items.length };
}
function checkHeap(heap: HeapView) { return heap.items.length === heap.len && heap.items.every((item, i) => i === 0 || heap.items[Math.floor((i - 1) / 2)].value >= item.value); }
function heapEventLabel(event: HeapTraceEvent): string {
  switch (event.kind) {
    case 'inserted': return `插入 ${event.value} 到索引 ${event.index}`;
    case 'compared': return `比较索引 ${event.parentIndex} 与 ${event.childIndex}：${event.ordering === 'less' ? '父节点较小' : '父节点符合顺序'}`;
    case 'comparedChildren': return `比较左右孩子 ${event.leftIndex} 与 ${event.rightIndex}，确定较大者`;
    case 'swapped': return `交换索引 ${event.firstIndex} 和 ${event.secondIndex}`;
    case 'rootRemoved': return `移除根节点 ${event.value}`;
    case 'accessed': return `读取最大值 ${event.value}`;
    case 'empty': return '堆为空';
  }
}
function heapEventExplanation(event: HeapTraceEvent) {
  switch (event.kind) {
    case 'inserted': return `新值 ${event.value} 先追加到数组末尾索引 ${event.index}，接下来沿父节点方向检查堆序。`;
    case 'compared': return `索引 ${event.parentIndex} 的父节点与索引 ${event.childIndex} 的子节点比较，Rust 判断父节点${event.ordering === 'less' ? '较小，需要交换' : '已满足最大堆顺序'}。`;
    case 'comparedChildren': return `下沉时先比较两个孩子，索引 ${event.ordering === 'leftGreaterOrEqual' ? event.leftIndex : event.rightIndex} 的值较大或相等，下一步拿它与父节点比较。`;
    case 'swapped': return `两个节点交换位置；下一步从新的父节点位置继续检查。`;
    case 'rootRemoved': return event.replacementId ? `最大值 ${event.value} 被移除，原末尾元素移动到根，随后向下恢复堆序。` : `唯一的元素 ${event.value} 被移除，堆变为空。`;
    case 'accessed': return `根节点 ${event.value} 是当前最大值；peek 不修改堆。`;
    case 'empty': return '堆为空，没有元素可读取或移除。';
  }
}
function heapPseudo(operation: HeapOperation) { return operation === 'push' ? ['将新值追加到数组末尾', '比较新节点与父节点', '若子节点更大则交换', '重复直到满足最大堆顺序'] : operation === 'pop' ? ['保存根节点最大值', '将最后一个元素移到根', '选择较大的子节点比较', '若子节点更大则交换并继续'] : ['检查堆是否为空', '读取索引 0 的根节点', '返回最大值，不修改堆']; }
function heapPseudoLine(event: HeapTraceEvent) { if (event.kind === 'inserted') return 0; if (event.kind === 'compared' || event.kind === 'comparedChildren') return 1; if (event.kind === 'swapped') return 2; if (event.kind === 'rootRemoved') return 1; if (event.kind === 'accessed' || event.kind === 'empty') return 0; return -1; }

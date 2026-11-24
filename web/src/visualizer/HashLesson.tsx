import { useEffect, useRef, useState } from 'react';
import type { HashTableView, HashTraceView, VisualizerCommand } from '../types';
import { getAvailableLesson } from './catalog';
import { getVisualizerAdapter } from './adapters';
import type { VisualizerSession } from './session';
import LessonHeader from './LessonHeader';

type HashStructureId = 'hash-map' | 'hash-set';

export default function HashLesson({ structureId }: { structureId: HashStructureId }) {
  const lesson = getAvailableLesson(structureId)!;
  const isMap = structureId === 'hash-map';
  const [view, setView] = useState<HashTableView>({ kind: 'hashTable', mode: isMap ? 'map' : 'set', buckets: [], len: 0, capacity: 0 });
  const [operation, setOperation] = useState('insert');
  const [key, setKey] = useState('');
  const [value, setValue] = useState('');
  const [trace, setTrace] = useState<HashTraceView | null>(null);
  const [cursor, setCursor] = useState(0);
  const [error, setError] = useState('');
  const session = useRef<VisualizerSession<HashTableView, HashTraceView> | null>(null);
  const event = trace?.events[Math.min(cursor, Math.max(0, trace.events.length - 1))] ?? null;
  const shown = trace
    ? cursor >= trace.events.length || event?.kind === 'inserted' || event?.kind === 'removed'
      ? trace.after
      : trace.before
    : view;

  useEffect(() => {
    let live = true;
    void getVisualizerAdapter<HashTableView, HashTraceView>(structureId).initialize()
      .then((nextSession) => {
        if (!live) return;
        session.current = nextSession;
        setView(nextSession.snapshot());
      })
      .catch((reason: unknown) => setError(`WASM 加载失败：${String(reason)}`));
    return () => { live = false; };
  }, [structureId]);

  function run() {
    try {
      if (!session.current) throw new Error('结构仍在加载，请稍后再试。');
      const int32 = (raw: string) => {
        const number = Number(raw);
        if (!raw.trim() || !Number.isInteger(number) || number < -2147483648 || number > 2147483647) {
          throw new Error('请输入有效的 32 位整数。');
        }
        return number;
      };
      const input: VisualizerCommand['input'] = operation === 'insert' && isMap
        ? { kind: 'keyValue', key: int32(key), value: int32(value) }
        : { kind: 'integerKey', value: int32(key) };
      setTrace(session.current.run({ structureId, operation, input }));
      setCursor(0);
      setError('');
    } catch (reason) {
      setError(String(reason));
    }
  }

  function loadExample(example: boolean) {
    const next = example ? session.current?.loadExample() : session.current?.reset();
    if (next) {
      setView(next);
      setTrace(null);
      setCursor(0);
      setError('');
    }
  }

  return <main className="app-shell hash-lesson-shell">
    <LessonHeader selectedId={structureId} />

    <div className="lesson-layout hash-layout">
      <section className="tree-card hash-table-card">
        <div className="tree-card-heading">
          <div><span className="section-kicker">HASH TABLE</span><h1>{lesson.label}</h1></div>
          <span className="hash-summary">{shown.len} 项 <i /> {shown.buckets.length} 个桶 <i /> 容量 {shown.capacity}</span>
        </div>
        <div className="hash-grid">
          {shown.buckets.map((bucket) => <div
            className={`hash-bucket ${event && 'bucket' in event && event.bucket === bucket.index ? 'active' : ''}`}
            key={bucket.index}
          >
            <strong>桶 {bucket.index}</strong>
            <div>{bucket.entries.map((entry) => <span className="hash-entry" key={entry.id}>
              {entry.key}{entry.value !== null ? ` → ${entry.value}` : ''}
            </span>)}{!bucket.entries.length && <i>空</i>}</div>
          </div>)}
        </div>
        <div className="tree-legend">桶编号由当前演示哈希器决定，顺序不代表键的大小；映射到同一桶的键会形成冲突链。</div>
      </section>

      <aside className="side-column hash-side-column">
        <section className="panel-card hash-panel">
          <div className="panel-title"><span className="section-kicker">TRY IT YOURSELF</span><h2>执行操作</h2></div>
          <label className="field-label" htmlFor="hash-operation">操作</label>
          <select id="hash-operation" value={operation} onChange={(e) => setOperation(e.target.value)}>
            {lesson.operations.map((item) => <option value={item.id} key={item.id}>{item.label}</option>)}
          </select>
          <div className={`hash-input-grid ${isMap && operation === 'insert' ? 'map-insert' : ''}`}>
            <label className="hash-field" htmlFor="hash-key">整数键
              <input id="hash-key" type="number" step="1" inputMode="numeric" value={key} placeholder="例如 42" onChange={(e) => setKey(e.target.value)} onKeyDown={(e) => { if (e.key === 'Enter') run(); }} />
            </label>
            {isMap && operation === 'insert' && <label className="hash-field" htmlFor="hash-value">整数值
              <input id="hash-value" type="number" step="1" inputMode="numeric" value={value} placeholder="例如 420" onChange={(e) => setValue(e.target.value)} onKeyDown={(e) => { if (e.key === 'Enter') run(); }} />
            </label>}
          </div>
          <button className="primary-button hash-run" onClick={run} disabled={!view.buckets.length}>运行操作 <span>↗</span></button>
          <div className="button-row"><button className="text-button" onClick={() => loadExample(true)}>载入示例</button><button className="text-button" onClick={() => loadExample(false)}>清空</button></div>
          {error && <p className="error-message" role="alert">{error}</p>}
          <div className="divider" />
          <div className="playback-heading"><span className="section-kicker">PLAYBACK</span><span>{trace ? `${Math.min(cursor + 1, trace.events.length)} / ${trace.events.length}` : '等待操作'}</span></div>
          <div className="playback-controls">
            <button className="icon-button" onClick={() => setCursor((current) => Math.max(0, current - 1))} disabled={!trace || cursor === 0}>←</button>
            <button className="icon-button" onClick={() => setCursor((current) => Math.min(trace?.events.length ?? 0, current + 1))} disabled={!trace || cursor >= trace.events.length}>→</button>
            <button className="icon-button" onClick={() => setCursor(0)} disabled={!trace || cursor === 0}>↻</button>
          </div>
        </section>

        <section className="panel-card hash-panel hash-step-panel">
          <div className="panel-title compact"><span className="section-kicker">ALGORITHM STEP</span><span className="step-number">{trace ? `${Math.min(cursor + 1, trace.events.length)} / ${trace.events.length}` : '—'}</span></div>
          <h2>{event ? eventLabel(event) : '准备开始'}</h2>
          <p>{event ? eventExplanation(event) : '选择操作，观察键如何映射到桶，并查看冲突与扩容。'}</p>
        </section>
      </aside>
    </div>
  </main>;
}

function eventLabel(event: HashTraceView['events'][number]) {
  switch (event.kind) {
    case 'compared': return `比较键 ${event.key} 与 ${event.comparedKey}`;
    case 'collision': return `发生冲突：桶中已有 ${event.chainLen} 项`;
    case 'inserted': return `插入键 ${event.key}`;
    case 'updated': return `更新键 ${event.key}`;
    case 'removed': return `删除键 ${event.key}`;
    case 'missing': return `未找到键 ${event.key}`;
    case 'resized': return `桶数量从 ${event.oldCount} 扩展到 ${event.newCount}`;
    case 'rehashed': return `键 ${event.key} 从桶 ${event.oldBucket} 移到桶 ${event.newBucket}`;
    case 'found': return `找到键 ${event.key}`;
    case 'duplicate': return `键 ${event.key} 已存在`;
  }
}

function eventExplanation(event: HashTraceView['events'][number]) {
  if (event.kind === 'compared') return event.matched ? '当前键匹配，查找结束。' : '当前项不匹配，继续检查冲突链。';
  if (event.kind === 'resized') return '负载达到容量阈值后，哈希表分配更多桶。';
  if (event.kind === 'rehashed') return '桶数改变后，键的目标桶需要重新计算。';
  if (event.kind === 'collision') return '多个不同键映射到同一桶，需要逐项比较。';
  return '此步骤由 Rust 哈希表操作生成。';
}

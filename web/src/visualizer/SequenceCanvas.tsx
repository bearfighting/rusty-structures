import type { SequenceTraceEvent, SequenceView } from '../types';

type Props = { sequence: SequenceView; currentEvent: SequenceTraceEvent | null };

export default function SequenceCanvas({ sequence, currentEvent }: Props) {
  const name = sequence.structure === 'stack' ? 'Stack' : sequence.structure === 'queue' ? 'Queue' : sequence.structure === 'deque' ? 'Deque' : 'Vector';
  return <div className={`sequence-visualization ${sequence.structure ?? 'vector'}`} role="group" aria-label={`${name}，共 ${sequence.len} 个元素${sequence.capacity === undefined ? '' : `，容量 ${sequence.capacity}`}`}>
    {currentEvent && <div className="tree-step-banner" aria-live="polite">
      <span>当前步骤</span><strong>{eventLabel(currentEvent, sequence)}</strong>
    </div>}
    <div className="sequence-stats">
      <span>长度 <strong>{sequence.len}</strong></span>
      {sequence.capacity !== undefined && <><span>容量 <strong>{sequence.capacity}</strong></span>
      <div className="capacity-track" role="img" aria-label={`已使用 ${sequence.len} / ${sequence.capacity} 个容量`}>
        <i style={{ width: `${sequence.capacity ? Math.min(100, sequence.len / sequence.capacity * 100) : 0}%` }} />
      </div></>}
    </div>
    <div className="sequence-slots" role="list" aria-label={`${name} 元素`}>
      {sequence.items.length ? sequence.items.map((item, index) => {
        const shift = currentEvent?.kind === 'shift' && currentEvent.itemId === item.id ? currentEvent : null;
        const inserted = currentEvent?.kind === 'inserted' && currentEvent.itemId === item.id;
        const removed = currentEvent?.kind === 'removed' && currentEvent.itemId === item.id;
        const accessed = currentEvent?.kind === 'accessed' && currentEvent.itemId === item.id;
        const classes = [
          'sequence-slot',
          shift ? `shifting-${shift.direction}` : '',
          inserted ? 'sequence-inserted' : '',
          removed ? 'sequence-removed' : '',
          accessed ? 'sequence-accessed' : '',
        ].filter(Boolean).join(' ');
        return <div className={classes} key={item.id} role="listitem" aria-label={`索引 ${index}，值 ${item.value}`}>
          <span className="slot-index">{index}</span>
          <strong>{item.value}</strong>
          {sequence.structure === 'stack' && index === 0 && <span className="sequence-end-label">TOP</span>}
          {sequence.structure === 'queue' && index === 0 && <span className="sequence-end-label">FRONT</span>}
          {sequence.structure === 'deque' && (index === 0 || index === sequence.len - 1) && <span className="sequence-end-label">{sequence.len === 1 ? 'FRONT / BACK' : index === 0 ? 'FRONT' : 'BACK'}</span>}
          {shift && <span className="shift-arrow" aria-hidden="true">{shift.direction === 'right' ? '→' : '←'}</span>}
        </div>;
      }) : <div className="sequence-empty">{name} 为空，可以使用上方操作添加元素。</div>}
    </div>
    {currentEvent?.kind === 'capacityChanged' && <p className="capacity-change">容量从 {currentEvent.oldCapacity} 扩展到 {currentEvent.newCapacity}</p>}
  </div>;
}

function eventLabel(event: SequenceTraceEvent, sequence: SequenceView): string {
  const name = sequence.structure === 'stack' ? 'Stack' : sequence.structure === 'queue' ? 'Queue' : sequence.structure === 'deque' ? 'Deque' : 'Vector';
  switch (event.kind) {
    case 'capacityChanged': return `扩展容量：${event.oldCapacity} → ${event.newCapacity}`;
    case 'shift': return `将值 ${event.value} 从索引 ${event.fromIndex} 向${event.direction === 'right' ? '右' : '左'}移动到 ${event.toIndex}`;
    case 'inserted': return sequence.structure === 'stack' ? `推入栈顶 ${event.value}` : sequence.structure === 'queue' ? `从队尾入队 ${event.value}` : sequence.structure === 'deque' ? `从${event.index === 0 ? '前' : '后'}端加入 ${event.value}` : `在索引 ${event.index} 插入 ${event.value}`;
    case 'removed': return sequence.structure === 'stack' ? `弹出栈顶 ${event.value}` : sequence.structure === 'queue' ? `从队首出队 ${event.value}` : sequence.structure === 'deque' ? `从${event.index === 0 ? '前' : '后'}端移除 ${event.value}` : `移除索引 ${event.index} 的 ${event.value}`;
    case 'accessed': return sequence.structure === 'stack' ? `查看栈顶 ${event.value}` : sequence.structure === 'queue' ? `${event.index === 0 ? '查看队首' : '查看队尾'} ${event.value}` : sequence.structure === 'deque' ? `查看${event.index === 0 ? '前' : '后'}端 ${event.value}` : `读取索引 ${event.index}，值为 ${event.value}`;
    case 'missingIndex': return `索引 ${event.index} 越界（长度 ${event.len}）`;
    case 'empty': return `${name} 为空，没有可访问或移除的元素`;
    case 'reserveUnchanged': return `容量足够，仍为 ${event.capacity}`;
  }
}

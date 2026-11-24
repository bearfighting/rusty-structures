import type { HeapTraceEvent, HeapView } from '../types';

export default function HeapCanvas({ heap, event }: { heap: HeapView; event: HeapTraceEvent | null }) {
  const active = event?.kind === 'compared' ? [event.parentId, event.childId]
    : event?.kind === 'comparedChildren' ? [event.leftId, event.rightId]
    : event?.kind === 'swapped' ? [event.firstId, event.secondId]
    : event?.kind === 'inserted' ? [event.itemId]
    : event?.kind === 'rootRemoved' || event?.kind === 'accessed' ? [event.itemId] : [];
  const nodes = heap.items.map((item) => {
    const depth = Math.floor(Math.log2(item.index + 1));
    const offset = item.index - (2 ** depth - 1);
    const count = 2 ** depth;
    const x = 400 + ((offset + 0.5) / count - 0.5) * 700;
    return { ...item, x, y: 48 + depth * 90 };
  });
  const byIndex = new Map(nodes.map((item) => [item.index, item]));
  return <div className="heap-visualization" role="group" aria-label={`最大堆，共 ${heap.len} 个元素`}>
    {event && <div className="tree-step-banner" aria-live="polite"><span>当前步骤</span><strong>{label(event)}</strong></div>}
    <svg className="heap-tree" viewBox="0 0 800 360" role="img" aria-label="二叉堆树形布局">
      {nodes.map((node) => {
        if (node.index === 0) return null;
        const parent = byIndex.get(Math.floor((node.index - 1) / 2));
        return parent ? <line key={`edge-${node.id}`} x1={parent.x} y1={parent.y + 25} x2={node.x} y2={node.y - 25} className="heap-edge" /> : null;
      })}
      {nodes.map((node) => <g key={node.id} className={`heap-node ${active.includes(node.id) ? 'active' : ''}`} transform={`translate(${node.x} ${node.y})`}>
        <circle r="25" /><text textAnchor="middle" dominantBaseline="central">{node.value}</text><text className="heap-index" textAnchor="middle" y="42">[{node.index}]</text>
      </g>)}
      {!nodes.length && <text x="400" y="160" textAnchor="middle" className="heap-empty">堆为空，可以插入一个整数。</text>}
    </svg>
    <div className="heap-array" aria-label="堆的数组布局">{heap.items.map((item) => <div key={item.id} className={active.includes(item.id) ? 'active' : ''}><small>{item.index}</small><strong>{item.value}</strong></div>)}</div>
  </div>;
}

function label(event: HeapTraceEvent) {
  switch (event.kind) {
    case 'inserted': return `将 ${event.value} 放入数组索引 ${event.index}`;
    case 'compared': return `比较父节点 ${event.parentIndex} 和子节点 ${event.childIndex}：${event.ordering === 'less' ? '父节点较小' : '父节点已不小于子节点'}`;
    case 'comparedChildren': return `比较左右子节点 ${event.leftIndex} 和 ${event.rightIndex}，选择较大的孩子`;
    case 'swapped': return `交换索引 ${event.firstIndex} 和 ${event.secondIndex}`;
    case 'rootRemoved': return event.replacementId ? `移除根节点 ${event.value}，将最后一个元素移到根` : `移除最后一个根节点 ${event.value}`;
    case 'accessed': return `读取根节点最大值 ${event.value}`;
    case 'empty': return '堆为空';
  }
}

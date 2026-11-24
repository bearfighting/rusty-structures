import type { LinkedListTraceEvent, LinkedListView } from '../types';

type Props = { list: LinkedListView; currentEvent: LinkedListTraceEvent | null; step: number; total: number };

export default function LinkedListCanvas({ list, currentEvent, step, total }: Props) {
  return <div className="linked-list-visualization" role="group" aria-label={`单向链表，共 ${list.len} 个节点`}>
    {currentEvent && <div className="tree-step-banner" aria-live="polite"><span>步骤 {step} / {total}</span><strong>{eventLabel(currentEvent, list)}</strong></div>}
    <div className="list-endpoints"><span>HEAD <strong>{list.headId ? valueForId(list, list.headId) : '∅'}</strong></span><span>TAIL <strong>{list.tailId ? valueForId(list, list.tailId) : '∅'}</strong></span></div>
    {list.nodes.length ? <div className="linked-list-track" role="list" aria-label="链表节点与 next 链接">
      {list.nodes.map((node, index) => {
        const isCreated = currentEvent?.kind === 'nodeCreated' && currentEvent.nodeId === node.id;
        const isRemoved = currentEvent?.kind === 'nodeRemoved' && currentEvent.nodeId === node.id;
        const isVisited = currentEvent?.kind === 'nodeVisited' && currentEvent.nodeId === node.id;
        const isHeadChange = currentEvent?.kind === 'headChanged' && currentEvent.toId === node.id;
        const isTailChange = currentEvent?.kind === 'tailChanged' && currentEvent.toId === node.id;
        return <div className="linked-list-unit" key={node.id} role="listitem">
          <div className={`linked-node ${isCreated ? 'linked-node-created' : ''} ${isRemoved ? 'linked-node-removed' : ''} ${isVisited ? 'linked-node-visited' : ''} ${isHeadChange || isTailChange ? 'linked-node-pointer-change' : ''}`}>
            <div className="linked-node-badges">{node.id === list.headId && <span>HEAD</span>}{node.id === list.tailId && <span>TAIL</span>}</div>
            <strong>{node.value}</strong><small>node</small>
          </div>
          {index < list.nodes.length - 1 && (() => {
            const nextNode = list.nodes[index + 1];
            const nextChanged = (currentEvent?.kind === 'linkCreated' || currentEvent?.kind === 'linkRemoved')
              && currentEvent.direction === 'next'
              && currentEvent.fromId === node.id
              && currentEvent.toId === nextNode.id;
            const prevChanged = list.linkMode === 'doubly'
              && (currentEvent?.kind === 'linkCreated' || currentEvent?.kind === 'linkRemoved')
              && currentEvent.direction === 'prev'
              && currentEvent.fromId === nextNode.id
              && currentEvent.toId === node.id;
            return <div className={`linked-arrow ${list.linkMode === 'doubly' ? 'linked-arrow-double' : ''} ${nextChanged || prevChanged ? 'linked-arrow-current' : ''}`} aria-label={`${list.linkMode === 'doubly' ? 'next 与 prev' : 'next'} 链接，连接 ${node.value} 和 ${nextNode.value}`}>
              <span>next</span><i className={nextChanged ? 'link-direction-current' : ''}>→</i>
              {list.linkMode === 'doubly' && <><span>prev</span><i className={prevChanged ? 'link-direction-current' : ''}>←</i></>}
            </div>;
          })()}
          {index === list.nodes.length - 1 && <div className="linked-null"><span>next</span><i>∅</i></div>}
        </div>;
      })}
    </div> : <div className="sequence-empty linked-list-empty">链表为空，可从头部或尾部添加节点。</div>}
  </div>;
}

function valueForId(list: LinkedListView, id: string | null): number | string {
  return id ? list.nodes.find((node) => node.id === id)?.value ?? '∅' : '∅';
}

function eventLabel(event: LinkedListTraceEvent, list: LinkedListView): string {
  switch (event.kind) {
    case 'nodeCreated': return `创建节点 ${event.value}`;
    case 'linkCreated': return `建立 ${event.direction} 链接`;
    case 'linkRemoved': return `断开 ${event.direction} 链接`;
    case 'headChanged': return `HEAD 指向 ${valueForId(list, event.toId)}`;
    case 'tailChanged': return `TAIL 指向 ${valueForId(list, event.toId)}`;
    case 'nodeRemoved': return `移除节点 ${event.value}`;
    case 'nodeVisited': return `访问${event.endpoint === 'head' ? '头部' : '尾部'}节点 ${event.value}`;
    case 'empty': return '链表为空';
  }
}

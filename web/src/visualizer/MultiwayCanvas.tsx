import { useMemo } from 'react';
import type { MultiwayNodeView, MultiwayTraceEvent, MultiwayTreeView } from '../types';

type Point = { x: number; y: number; node: MultiwayNodeView };
export default function MultiwayCanvas({ tree, event }: { tree: MultiwayTreeView; event: MultiwayTraceEvent | null }) {
  const layout = useMemo(() => layoutTree(tree.root), [tree.root]);
  const active = event ? eventNodeIds(event) : [];
  const pointById=new Map(layout.points.map(point=>[point.node.id,point]));
  return <div className="multiway-visualization" role="group" aria-label={`B 树映射，共 ${tree.len} 个键，${tree.height} 层`}>
    {event && <div className="tree-step-banner"><span>当前步骤</span><strong>{eventLabel(event)}</strong></div>}
    {layout.points.length ? <svg className="multiway-svg" viewBox={`0 0 ${layout.width} ${layout.height}`} role="img" aria-label={tree.family==='bPlus'?'B+ 树，内部索引与叶链':'B 树多路节点布局'}>
      <defs><marker id="bplus-arrow" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="5" markerHeight="5" orient="auto-start-reverse"><path d="M 0 0 L 10 5 L 0 10 z" fill="#5e9888"/></marker></defs>
      {layout.edges.map((edge) => <line key={edge.key} x1={edge.x1} y1={edge.y1} x2={edge.x2} y2={edge.y2} className="multiway-edge"/>)}
      {tree.family==='bPlus'&&layout.points.map(({node,x,y})=>{const next=node.nextId?pointById.get(node.nextId):undefined;return next?<g key={`leaf-link-${node.id}`}><path d={`M ${x} ${y+48} C ${x} ${y+78}, ${next.x} ${next.y+78}, ${next.x} ${next.y+48}`} className="bplus-leaf-link"/><text x={(x+next.x)/2} y={Math.max(y,next.y)+79} textAnchor="middle" className="bplus-link-label">next</text></g>:null;})}
      {layout.points.map(({node,x,y}) => {
        const width = Math.max(66, node.entries.length * 68 + 12);
        return <g key={node.id} className={`multiway-node ${node.nodeKind==='leaf'?'leaf-node':''} ${node.nodeKind==='internal'?'internal-node':''} ${active.includes(node.id) ? 'active' : ''}`} transform={`translate(${x-width/2},${y})`}>
          {node.entries.map((entry,index) => <g key={entry.id} transform={`translate(${6+index*68},0)`}><rect width="62" height={entry.value===null?30:38} rx="7"/><text x="31" y={entry.value===null?19:15} textAnchor="middle">{entry.key}</text>{entry.value!==null&&<text className="multiway-value" x="31" y="30" textAnchor="middle">{entry.value}</text>}</g>)}
        </g>;
      })}
    </svg> : <div className="empty-canvas"><div className="empty-glyph">∅</div><strong>B 树为空</strong><span>插入键值对以开始。</span></div>}
  </div>;
}
function layoutTree(root: MultiwayNodeView | null) {
  const points: Point[]=[]; const edges:{key:string;x1:number;y1:number;x2:number;y2:number}[]=[]; let rank=0; let depthMax=0;
  function visit(node:MultiwayNodeView,depth:number):number {
    depthMax=Math.max(depthMax,depth); const xs:number[]=[];
    for(let i=0;i<node.entries.length;i++) { if(node.children[i]) xs.push(visit(node.children[i],depth+1)); xs.push(52+rank*84); rank++; }
    if(node.children[node.entries.length]) xs.push(visit(node.children[node.entries.length],depth+1));
    const x=xs.reduce((a,b)=>a+b,0)/xs.length; const y=52+depth*104; points.push({node,x,y});
    for(const child of node.children){const childPoint=points.find(p=>p.node.id===child.id)!;edges.push({key:`${node.id}-${child.id}`,x1:x,y1:y+40,x2:childPoint.x,y2:childPoint.y});}
    return x;
  }
  if(root) visit(root,0);
  return {points,edges,width:Math.max(560,rank*84+50),height:Math.max(180,depthMax*104+120)};
}
function eventNodeIds(event:MultiwayTraceEvent):string[]{switch(event.kind){case 'compared':return[event.nodeId];case 'split':return[event.parentId,event.leftId,event.rightId];case 'borrowed':return[event.parentId,event.childId,event.siblingId];case 'merged':return[event.parentId,event.leftId,event.rightId];case'leafSplit':return[event.leftId,event.rightId];case'internalSplit':case'separatorUpdated':return[event.nodeId];case'leafMerged':return[event.removedLeafId,event.intoLeafId];case'leafBorrowed':return[event.fromLeafId,event.toLeafId];case'leafLinked':return[event.fromId,...(event.toId?[event.toId]:[])];default:return[];}}
function eventLabel(e:MultiwayTraceEvent){switch(e.kind){case'compared':return`在节点中检查键 ${e.key}`;case'split':return`分裂节点并提升中位键 ${e.promotedKey}`;case'borrowed':return`向${e.direction==='left'?'左':'右'}兄弟借键 ${e.key}`;case'merged':return`合并节点并吸收分隔键 ${e.separatorKey}`;case'leafSplit':return`分裂叶节点，分隔键 ${e.separatorKey}`;case'internalSplit':return`分裂索引节点，分隔键 ${e.separatorKey}`;case'leafLinked':return`更新叶链：${e.fromId} → ${e.toId??'结束'}`;case'leafMerged':return`合并叶节点`;case'leafBorrowed':return`叶节点间移动键 ${e.key}`;case'separatorUpdated':return`更新分隔键 ${e.oldKey} → ${e.newKey}`;case'scanEntry':return`按叶链访问 ${e.key}`;case'inserted':return e.value===null?`插入元素 ${e.key}`:`插入 ${e.key} → ${e.value}`;case'updated':return`更新键 ${e.key} 的值`;case'duplicate':return`${e.key} 已存在`;case'replacementSelected':return`用${e.direction==='predecessor'?'前驱':'后继'} ${e.replacement} 替换 ${e.key}`;case'found':return e.value===null?`找到元素 ${e.key}`:`找到键 ${e.key}，值 ${e.value}`;case'missing':return`没有键 ${e.key}`;case'removed':return e.value===null?`移除元素 ${e.key}`:`删除 ${e.key} → ${e.value}`;}}

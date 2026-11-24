import { useMemo } from 'react';
import type { TraceEvent, TreeView } from '../types';
import { layoutTree } from './tree-model';

type Props = {
  tree: TreeView;
  activeIds: Set<string>;
  currentEvent: TraceEvent | null;
  stepNumber: number;
  stepCount: number;
  selected: string | null;
  onSelect: (id: string) => void;
};

export default function TreeCanvas({ tree, activeIds, currentEvent, stepNumber, stepCount, selected, onSelect }: Props) {
  const layout = useMemo(() => layoutTree(tree.root), [tree.root]);
  return <div className="tree-visualization">
    {currentEvent && <div className="tree-step-banner" aria-live="polite">
      <span>步骤 {stepNumber} / {stepCount}</span><strong>{canvasStepLabel(currentEvent)}</strong>
    </div>}
    {tree.root && <svg className="tree-svg" viewBox={`0 0 ${layout.width} ${layout.height}`} role="img" aria-label={`二叉搜索树，共 ${tree.len} 个节点`}>
    <g className="edges">{layout.edges.map((edge) => {
      const currentBranch = currentEvent?.kind === 'descend'
        && currentEvent.fromId === edge.parent
        && currentEvent.direction === edge.direction;
      const visitedPath = activeIds.has(edge.parent) && activeIds.has(edge.child);
      const redLink = findMetrics(tree.root, edge.child)?.color === 'red';
      return <line key={edge.key} x1={edge.x1} y1={edge.y1} x2={edge.x2} y2={edge.y2} className={`${redLink ? 'red-edge' : ''} ${visitedPath ? 'path-edge' : ''} ${currentBranch ? 'current-edge' : ''}`} />;
    })}</g>
    <g className="nodes">{layout.nodes.map((node) => {
      const comparing = currentEvent?.kind === 'compare' && currentEvent.nodeId === node.id;
      const inserting = currentEvent?.kind === 'inserted' && currentEvent.nodeId === node.id;
      const removing = currentEvent?.kind === 'removed' && currentEvent.nodeId === node.id;
      const metrics = findMetrics(tree.root, node.id);
      return <g key={node.id} className={`tree-node ${metrics?.color === 'red' ? 'red-node' : ''} ${activeIds.has(node.id) ? 'active' : ''} ${comparing ? 'comparing' : ''} ${inserting ? 'entering' : ''} ${removing ? 'removing' : ''} ${selected === node.id ? 'selected' : ''}`} transform={`translate(${node.x}, ${node.y})`} onClick={() => onSelect(node.id)} onKeyDown={(event) => { if (event.key === 'Enter' || event.key === ' ') onSelect(node.id); }} role="button" tabIndex={0} aria-label={`节点 ${node.value}`}>
      {comparing && <circle className="compare-halo" r="35" />}
      {removing && <circle className="remove-halo" r="35" />}
      <circle r="26" /><text textAnchor="middle" dominantBaseline="central">{node.value}</text>
      {metrics?.balanceFactor !== undefined && <text className="tree-metrics" textAnchor="middle" y="43">h{metrics.height} · bf{metrics.balanceFactor}</text>}
      {metrics?.color && <text className="tree-color-label" textAnchor="middle" y="43">{metrics.color === 'red' ? '红色' : '黑色'}</text>}
    </g>;
    })}</g>
    {!currentEvent && <text className="root-label" x={layout.nodes.find((node) => node.id === tree.root?.id)?.x} y="22" textAnchor="middle">ROOT</text>}
    </svg>}
    {!tree.root && <div className="empty-canvas"><div className="empty-glyph">∅</div><strong>树是空的</strong><span>插入一个整数，开始构建你的树。</span></div>}
  </div>;
}

function findMetrics(root: TreeView['root'], id: string): TreeView['root'] {
  if (!root) return null;
  if (root.id === id) return root;
  return findMetrics(root.left, id) ?? findMetrics(root.right, id);
}

function canvasStepLabel(event: TraceEvent): string {
  switch (event.kind) {
    case 'compare': return `比较 ${event.key} 与节点 ${event.nodeValue}`;
    case 'descend': return `沿${event.direction === 'left' ? '左' : '右'}分支继续`;
    case 'inserted': return `插入节点 ${event.key}`;
    case 'duplicate': return `${event.key} 已存在`;
    case 'found': return `找到节点 ${event.key}`;
    case 'missing': return `未找到 ${event.key}`;
    case 'successorSelected': return `选择 ${event.successor} 作为中序后继`;
    case 'removed': return `移除节点 ${event.key}`;
  }
}

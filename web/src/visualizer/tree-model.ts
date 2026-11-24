import type { NodeView, TraceEvent, TraceView, TreeView } from '../types';

export type Point = { id: string; value: number; x: number; y: number };
export type Edge = { key: string; parent: string; child: string; direction: 'left' | 'right'; x1: number; y1: number; x2: number; y2: number };

export function layoutTree(root: NodeView | null) {
  const nodes: Point[] = [];
  const edges: Edge[] = [];
  let rank = 0;
  let maxDepth = 0;

  function visit(node: NodeView | null, depth: number): void {
    if (!node) return;
    visit(node.left, depth + 1);
    nodes.push({ id: node.id, value: node.value, x: 72 + rank * 104, y: 62 + depth * 104 });
    rank += 1;
    maxDepth = Math.max(maxDepth, depth);
    visit(node.right, depth + 1);
  }
  visit(root, 0);

  const byId = new Map(nodes.map((node) => [node.id, node]));
  function connect(node: NodeView | null): void {
    if (!node) return;
    const parent = byId.get(node.id)!;
    for (const { child, direction } of [{ child: node.left, direction: 'left' as const }, { child: node.right, direction: 'right' as const }]) {
      if (!child) continue;
      const point = byId.get(child.id)!;
      edges.push({ key: `${node.id}-${child.id}`, parent: node.id, child: child.id, direction, x1: parent.x, y1: parent.y + 26, x2: point.x, y2: point.y - 26 });
      connect(child);
    }
  }
  connect(root);
  return { nodes, edges, width: Math.max(520, 144 + Math.max(0, rank - 1) * 104), height: Math.max(180, 108 + maxDepth * 104) };
}

export function getActiveIds(trace: TraceView | null, cursor: number) {
  const ids = new Set<string>();
  if (!trace || cursor >= trace.events.length) return ids;
  for (const event of trace.events.slice(0, cursor + 1)) {
    if (event.kind === 'compare') ids.add(event.nodeId);
    if (event.kind === 'descend') ids.add(event.fromId);
    if (event.kind === 'inserted') ids.add(event.nodeId);
    if (event.kind === 'successorSelected') { ids.add(event.removedId); ids.add(event.successorId); }
    if (event.kind === 'removed') ids.add(event.nodeId);
  }
  return ids;
}

export function checkTree(tree: TreeView) {
  let count = 0;
  function visit(node: NodeView | null, min: number | null, max: number | null): boolean {
    if (!node) return true;
    count += 1;
    if ((min !== null && node.value <= min) || (max !== null && node.value >= max)) return false;
    return visit(node.left, min, node.value) && visit(node.right, node.value, max);
  }
  const ordered = visit(tree.root, null, null);
  return { ok: ordered && count === tree.len, count, message: !ordered ? '节点没有遵循左小右大的顺序。' : `节点数不一致：${count} / ${tree.len}` };
}

export function treeHeight(root: NodeView | null): number {
  return root ? 1 + Math.max(treeHeight(root.left), treeHeight(root.right)) : 0;
}

export function nodeRelation(root: NodeView | null, id: string): string {
  function find(node: NodeView | null, parent: NodeView | null): string | null {
    if (!node) return null;
    if (node.id === id) return !parent ? '根节点' : parent.left?.id === id ? `节点 ${parent.value} 的左孩子` : `节点 ${parent.value} 的右孩子`;
    return find(node.left, node) ?? find(node.right, node);
  }
  return find(root, null) ?? '节点不在当前画面中';
}

export type { TraceEvent };

export type TreeView = { root: NodeView | null; len: number };
export type NodeView = {
  id: string;
  value: number;
  left: NodeView | null;
  right: NodeView | null;
};
export type TraceView = {
  before: TreeView;
  after: TreeView;
  events: TraceEvent[];
};
export type TraceEvent =
  | { kind: 'compare'; nodeId: string; nodeValue: number; key: number; ordering: 'less' | 'equal' | 'greater' }
  | { kind: 'descend'; fromId: string; direction: 'left' | 'right' }
  | { kind: 'inserted'; nodeId: string; key: number }
  | { kind: 'duplicate'; key: number }
  | { kind: 'found'; key: number }
  | { kind: 'missing'; key: number }
  | { kind: 'successorSelected'; removed: number; successor: number }
  | { kind: 'removed'; key: number };

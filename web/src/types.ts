export type TreeView = { kind: 'binaryTree'; root: NodeView | null; len: number };
export type SequenceView = {
  kind: 'sequence';
  structure?: 'vector'|'stack'|'queue'|'deque';
  items: VisualItem[];
  len: number;
  capacity?: number;
  frontIndex?: number;
};
export type VisualItem = { id: string; label: string; value: string | number };
export type StructureView = TreeView | SequenceView | LinkedListView | HeapView | PriorityQueueView | MultiwayTreeView | HashTableView | TreeMapView | TrieView;
export type PriorityQueueView = {kind:'priorityQueue';items:PriorityQueueItemView[];len:number};
export type PriorityQueueItemView = {id:string;priority:number;value:number;index:number};
export type PriorityQueueTraceEvent = {kind:'inserted';itemId:string;index:number;priority:number;value:number}|{kind:'compared';parentId:string;childId:string;parentIndex:number;childIndex:number;parentPriority:number;childPriority:number;ordering:'less'|'greaterOrEqual'}|{kind:'comparedChildren';leftId:string;rightId:string;leftIndex:number;rightIndex:number;leftPriority:number;rightPriority:number;ordering:'leftGreaterOrEqual'|'rightGreater'}|{kind:'swapped';firstId:string;secondId:string;firstIndex:number;secondIndex:number}|{kind:'rootRemoved';itemId:string;priority:number;value:number;replacementId:string|null;replacementPriority:number|null;replacementValue:number|null}|{kind:'accessed';itemId:string;priority:number;value:number}|{kind:'empty'};
export type PriorityQueueTraceView = {before:PriorityQueueView;after:PriorityQueueView;events:PriorityQueueTraceEvent[]};
export type TrieView = {kind:'trie';root:TrieNodeView;len:number};
export type TrieNodeView = {id:string;prefix:string;edgeLabel:string|null;isWord:boolean;children:TrieNodeView[]};
export type TrieTraceEvent = {kind:'visited';nodeId:string;prefix:string;character:string|null;matched:boolean}|{kind:'nodeCreated';nodeId:string;prefix:string;character:string}|{kind:'wordMarked';nodeId:string;word:string}|{kind:'terminalCleared';nodeId:string;word:string}|{kind:'nodePruned';nodeId:string;prefix:string}|{kind:'duplicate';word:string}|{kind:'found';word:string}|{kind:'prefixFound';prefix:string;matches:number}|{kind:'scanWord';word:string}|{kind:'missing';query:string}|{kind:'removed';word:string};
export type TrieTraceView = {before:TrieView;after:TrieView;events:TrieTraceEvent[]};
export type TreeMapView = {kind:'treeMap';backend:string;entries:TreeMapEntryView[];len:number};
export type TreeMapEntryView = {id:string;key:number;value:number};
export type TreeMapTraceEvent = {kind:'inserted';key:number;value:number}|{kind:'updated';key:number;oldValue:number;value:number}|{kind:'found';key:number;value:number}|{kind:'missing';key:number}|{kind:'removed';key:number;value:number}|{kind:'scanEntry';key:number;value:number};
export type TreeMapTraceView = {before:TreeMapView;after:TreeMapView;events:TreeMapTraceEvent[]};
export type HashTableView = { kind:'hashTable'; mode:'map'|'set'; buckets:HashBucketView[]; len:number; capacity:number };
export type HashBucketView = { index:number; entries:HashEntryView[] };
export type HashEntryView = { id:string; key:number; value:number|null };
export type HashTraceEvent = {kind:'compared';bucket:number;key:number;comparedKey:number;matched:boolean}|{kind:'collision';bucket:number;key:number;chainLen:number}|{kind:'inserted';bucket:number;key:number;value:number|null}|{kind:'updated';bucket:number;key:number;value:number}|{kind:'removed';bucket:number;key:number}|{kind:'missing';bucket:number;key:number}|{kind:'resized';oldCount:number;newCount:number}|{kind:'rehashed';key:number;oldBucket:number;newBucket:number}|{kind:'found';bucket:number;key:number;value:number|null}|{kind:'duplicate';bucket:number;key:number};
export type HashTraceView = {before:HashTableView;after:HashTableView;events:HashTraceEvent[]};
export type MultiwayTreeView = { kind: 'multiwayTree'; family: 'bTree' | 'bPlus'; mode: 'map' | 'set'; root: MultiwayNodeView | null; len: number; height: number; leafIds?: string[] };
export type MultiwayNodeView = { id: string; nodeKind: 'bTree' | 'internal' | 'leaf'; entries: MultiwayEntryView[]; children: MultiwayNodeView[]; nextId?: string | null };
export type MultiwayEntryView = { id: string; key: number; value: number | null };
export type HeapView = { kind: 'heap'; items: HeapItemView[]; len: number };
export type HeapItemView = { id: string; value: number; index: number };
export type LinkedListView = {
  kind: 'linkedList';
  linkMode: 'singly' | 'doubly';
  nodes: LinkedListNodeView[];
  headId: string | null;
  tailId: string | null;
  len: number;
};
export type LinkedListNodeView = { id: string; value: number; nextId: string | null; prevId: string | null };

export type OperationInput =
  | { kind: 'none' }
  | { kind: 'integerKey'; value: number }
  | { kind: 'value'; value: string | number }
  | { kind: 'index'; index: number }
  | { kind: 'indexedValue'; index: number; value: string | number }
  | { kind: 'capacity'; additional: number }
  | { kind: 'keyValue'; key: string | number; value: string | number }
  | { kind: 'word'; value: string };
export type VisualizerCommand = {
  structureId: string;
  operation: string;
  input: OperationInput;
};
export type NodeView = {
  id: string;
  value: number;
  left: NodeView | null;
  right: NodeView | null;
  height?: number;
  balanceFactor?: number;
  color?: 'red' | 'black';
};
export type AvlEvent =
  | { kind: 'compared'; nodeId: string; nodeValue: number; key: number; ordering: 'less' | 'equal' | 'greater' }
  | { kind: 'inserted'; nodeId: string; key: number }
  | { kind: 'duplicate'; key: number }
  | { kind: 'rotated'; direction: 'left' | 'right'; pivotId: string; rootId: string; case: 'LL' | 'LR' | 'RR' | 'RL' }
  | { kind: 'found'; nodeId: string; key: number }
  | { kind: 'missing'; key: number }
  | { kind: 'successorSelected'; removedId: string; successorId: string; removed: number; successor: number }
  | { kind: 'removed'; nodeId: string; key: number };
export type AvlTraceView = { before: TreeView; after: TreeView; events: AvlEvent[] };
export type RedBlackEvent =
  | { kind: 'compared'; nodeId: string; nodeValue: number; key: number; ordering: 'less' | 'equal' | 'greater' }
  | { kind: 'rotated'; direction: 'left' | 'right'; rootId: string; pivotId: string }
  | { kind: 'colorsFlipped'; nodeId: string; childIds: string[] }
  | { kind: 'redLinkMoved'; direction: 'left' | 'right'; nodeId: string }
  | { kind: 'inserted'; nodeId: string; key: number }
  | { kind: 'duplicate'; key: number }
  | { kind: 'successorSelected'; removedId: string; successorId: string; removed: number; successor: number }
  | { kind: 'found'; nodeId: string; key: number }
  | { kind: 'missing'; key: number }
  | { kind: 'removed'; nodeId: string; key: number };
export type RedBlackTraceView = { before: TreeView; after: TreeView; events: RedBlackEvent[] };
export type MultiwayTraceEvent =
  | { kind: 'compared'; nodeId: string; key: number; keyIndex: number; ordering: 'less' | 'equal' | 'greater' }
  | { kind: 'split'; parentId: string; leftId: string; rightId: string; promotedKey: number }
  | { kind: 'borrowed'; parentId: string; childId: string; siblingId: string; direction: 'left' | 'right'; key: number }
  | { kind: 'merged'; parentId: string; leftId: string; rightId: string; separatorKey: number }
  | { kind: 'inserted'; entryId: string; key: number; value: number | null }
  | { kind: 'updated'; entryId: string; key: number; oldValue: number | null; value: number | null }
  | { kind: 'duplicate'; key: number }
  | { kind: 'replacementSelected'; key: number; replacement: number; direction: 'predecessor' | 'successor' }
  | { kind: 'found'; key: number; value: number | null }
  | { kind: 'missing'; key: number }
  | { kind: 'removed'; entryId: string; key: number; value: number | null }
  | { kind: 'leafSplit'; leftId: string; rightId: string; separatorKey: number }
  | { kind: 'internalSplit'; nodeId: string; separatorKey: number }
  | { kind: 'leafLinked'; fromId: string; toId: string | null }
  | { kind: 'leafMerged'; removedLeafId: string; intoLeafId: string }
  | { kind: 'leafBorrowed'; fromLeafId: string; toLeafId: string; key: number }
  | { kind: 'separatorUpdated'; nodeId: string; keyIndex: number; oldKey: number; newKey: number }
  | { kind: 'scanEntry'; key: number; value: number | null; entryId: string };
export type MultiwayTraceView = { before: MultiwayTreeView; after: MultiwayTreeView; events: MultiwayTraceEvent[] };
export type TraceView = {
  before: TreeView;
  after: TreeView;
  events: TraceEvent[];
};
export type SequenceTraceView = {
  before: SequenceView;
  after: SequenceView;
  events: SequenceTraceEvent[];
};
export type LinkedListTraceView = {
  before: LinkedListView;
  after: LinkedListView;
  events: LinkedListTraceEvent[];
};
export type HeapTraceView = { before: HeapView; after: HeapView; events: HeapTraceEvent[] };
export type HeapTraceEvent =
  | { kind: 'inserted'; itemId: string; index: number; value: number }
  | { kind: 'compared'; parentId: string; childId: string; parentIndex: number; childIndex: number; ordering: 'less' | 'greaterOrEqual' }
  | { kind: 'comparedChildren'; leftId: string; rightId: string; leftIndex: number; rightIndex: number; ordering: 'leftGreaterOrEqual' | 'rightGreater' }
  | { kind: 'swapped'; firstId: string; secondId: string; firstIndex: number; secondIndex: number }
  | { kind: 'rootRemoved'; itemId: string; value: number; replacementId: string | null; replacementValue: number | null }
  | { kind: 'accessed'; itemId: string; value: number }
  | { kind: 'empty' };
export type LinkedListTraceEvent =
  | { kind: 'nodeCreated'; nodeId: string; value: number }
  | { kind: 'linkCreated'; direction: 'next' | 'prev'; fromId: string; toId: string }
  | { kind: 'linkRemoved'; direction: 'next' | 'prev'; fromId: string; toId: string }
  | { kind: 'headChanged'; fromId: string | null; toId: string | null }
  | { kind: 'tailChanged'; fromId: string | null; toId: string | null }
  | { kind: 'nodeRemoved'; nodeId: string; index: number; value: number }
  | { kind: 'nodeVisited'; nodeId: string; endpoint: 'head' | 'tail'; value: number }
  | { kind: 'empty' };
export type VisualizerTrace = TraceView | SequenceTraceView | LinkedListTraceView | HeapTraceView | PriorityQueueTraceView | AvlTraceView | RedBlackTraceView | MultiwayTraceView | HashTraceView | TreeMapTraceView | TrieTraceView;
export type SequenceTraceEvent =
  | { kind: 'capacityChanged'; oldCapacity: number; newCapacity: number }
  | { kind: 'shift'; direction: 'right' | 'left'; itemId: string; value: number; fromIndex: number; toIndex: number }
  | { kind: 'inserted'; itemId: string; index: number; value: number }
  | { kind: 'removed'; itemId: string; index: number; value: number }
  | { kind: 'accessed'; itemId: string; index: number; value: number }
  | { kind: 'missingIndex'; index: number; len: number }
  | { kind: 'empty' }
  | { kind: 'reserveUnchanged'; capacity: number };
export type TraceEvent =
  | { kind: 'compare'; nodeId: string; nodeValue: number; key: number; ordering: 'less' | 'equal' | 'greater' }
  | { kind: 'descend'; fromId: string; direction: 'left' | 'right' }
  | { kind: 'inserted'; nodeId: string; key: number }
  | { kind: 'duplicate'; key: number }
  | { kind: 'found'; nodeId: string; key: number }
  | { kind: 'missing'; key: number }
  | { kind: 'successorSelected'; removedId: string; successorId: string; removed: number; successor: number }
  | { kind: 'removed'; nodeId: string; key: number };

import type { OperationInput } from '../types';
import { visualizerAdapters } from './adapters';

export type ViewKind = 'sequence' | 'linkedList' | 'binaryTree' | 'multiwayTree' | 'heap' | 'priorityQueue' | 'hashTable' | 'treeMap' | 'trie';
export type CatalogStatus = 'available' | 'planned';
export type OperationDescriptor = {
  id: string;
  label: string;
  input: OperationInput['kind'];
};
export type LessonDescriptor = {
  id: string;
  category: string;
  label: string;
  implementation: string;
  status: CatalogStatus;
  viewKind: ViewKind;
  adapterId?: string;
  operations: readonly OperationDescriptor[];
};

const bstOperations: readonly OperationDescriptor[] = [
  { id: 'insert', label: '插入节点', input: 'integerKey' },
  { id: 'search', label: '查找节点', input: 'integerKey' },
  { id: 'remove', label: '删除节点', input: 'integerKey' },
];
const vectorOperations: readonly OperationDescriptor[] = [
  { id: 'push', label: '追加到末尾', input: 'value' },
  { id: 'pop', label: '移除末尾', input: 'none' },
  { id: 'insert', label: '按索引插入', input: 'indexedValue' },
  { id: 'remove', label: '按索引移除', input: 'index' },
  { id: 'get', label: '读取索引', input: 'index' },
  { id: 'reserve', label: '预留容量', input: 'capacity' },
];
const stackOperations: readonly OperationDescriptor[] = [{id:'push',label:'压入栈顶',input:'value'},{id:'pop',label:'弹出栈顶',input:'none'},{id:'peek',label:'查看栈顶',input:'none'}];
const queueOperations: readonly OperationDescriptor[] = [{id:'enqueue',label:'从队尾入队',input:'value'},{id:'dequeue',label:'从队首出队',input:'none'},{id:'front',label:'查看队首',input:'none'},{id:'back',label:'查看队尾',input:'none'}];
const dequeOperations: readonly OperationDescriptor[] = [{id:'pushFront',label:'从前端加入',input:'value'},{id:'pushBack',label:'从后端加入',input:'value'},{id:'popFront',label:'从前端移除',input:'none'},{id:'popBack',label:'从后端移除',input:'none'},{id:'front',label:'查看前端',input:'none'},{id:'back',label:'查看后端',input:'none'}];
const singlyLinkedListOperations: readonly OperationDescriptor[] = [
  { id: 'pushFront', label: '从头部添加', input: 'value' },
  { id: 'pushBack', label: '从尾部添加', input: 'value' },
  { id: 'popFront', label: '移除头部', input: 'none' },
  { id: 'front', label: '查看头部', input: 'none' },
  { id: 'back', label: '查看尾部', input: 'none' },
];
const doublyLinkedListOperations: readonly OperationDescriptor[] = [
  ...singlyLinkedListOperations,
  { id: 'popBack', label: '移除尾部', input: 'none' },
];
const heapOperations: readonly OperationDescriptor[] = [
  { id: 'push', label: '插入元素', input: 'value' },
  { id: 'pop', label: '移除最大值', input: 'none' },
  { id: 'peek', label: '查看最大值', input: 'none' },
];
const priorityQueueOperations: readonly OperationDescriptor[] = [{id:'push',label:'加入优先级和值',input:'keyValue'},{id:'pop',label:'移除最高优先级',input:'none'},{id:'peek',label:'查看最高优先级',input:'none'}];
const avlOperations: readonly OperationDescriptor[] = [
  { id: 'insert', label: '插入节点', input: 'integerKey' },
  { id: 'search', label: '查找节点', input: 'integerKey' },
  { id: 'remove', label: '删除节点', input: 'integerKey' },
];
const redBlackOperations = avlOperations;
const bTreeOperations: readonly OperationDescriptor[] = [
  { id: 'insert', label: '插入键值对', input: 'keyValue' },
  { id: 'search', label: '查找键', input: 'integerKey' },
  { id: 'remove', label: '删除键', input: 'integerKey' },
];
const bPlusMapOperations: readonly OperationDescriptor[] = [...bTreeOperations, { id: 'scan', label: '按叶链扫描', input: 'none' }];
const bPlusSetOperations: readonly OperationDescriptor[] = [{id:'insert',label:'插入元素',input:'integerKey'},{id:'search',label:'查找元素',input:'integerKey'},{id:'remove',label:'删除元素',input:'integerKey'},{id:'scan',label:'按叶链扫描',input:'none'}];
const hashMapOperations: readonly OperationDescriptor[] = [{id:'insert',label:'插入键值对',input:'keyValue'},{id:'search',label:'查找键',input:'integerKey'},{id:'remove',label:'删除键',input:'integerKey'}];
const hashSetOperations: readonly OperationDescriptor[] = [{id:'insert',label:'插入元素',input:'integerKey'},{id:'search',label:'查找元素',input:'integerKey'},{id:'remove',label:'删除元素',input:'integerKey'}];
const treeMapOperations: readonly OperationDescriptor[] = [...bTreeOperations, {id:'scan',label:'顺序扫描',input:'none'}];
const trieOperations: readonly OperationDescriptor[] = [{id:'insert',label:'插入单词',input:'word'},{id:'contains',label:'精确查找',input:'word'},{id:'startsWith',label:'前缀查找',input:'word'},{id:'remove',label:'删除单词',input:'word'},{id:'scan',label:'列出前缀匹配',input:'word'}];

const planned = (id: string, category: string, label: string, implementation: string, viewKind: ViewKind): LessonDescriptor => ({
  id, category, label, implementation, status: 'planned', viewKind, operations: [],
});

export const lessonCatalog: readonly LessonDescriptor[] = [
  { id: 'vector', category: '线性容器', label: '动态数组', implementation: 'Vector<i32>', status: 'available', viewKind: 'sequence', adapterId: 'vector', operations: vectorOperations },
  {id:'stack',category:'线性容器',label:'栈',implementation:'Stack<i32>',status:'available',viewKind:'sequence',adapterId:'stack',operations:stackOperations},
  {id:'queue',category:'线性容器',label:'队列',implementation:'Queue<i32>',status:'available',viewKind:'sequence',adapterId:'queue',operations:queueOperations},
  {id:'deque',category:'线性容器',label:'双端队列',implementation:'Deque<i32>',status:'available',viewKind:'sequence',adapterId:'deque',operations:dequeOperations},
  { id: 'singly-linked-list', category: '链表', label: '单向链表', implementation: 'SinglyLinkedList<i32>', status: 'available', viewKind: 'linkedList', adapterId: 'singly-linked-list', operations: singlyLinkedListOperations },
  { id: 'doubly-linked-list', category: '链表', label: '双向链表', implementation: 'DoublyLinkedList<i32>', status: 'available', viewKind: 'linkedList', adapterId: 'doubly-linked-list', operations: doublyLinkedListOperations },
  { id: 'binary-search-tree', category: '搜索树', label: '二叉搜索树', implementation: 'BinarySearchTree（迭代）', status: 'available', viewKind: 'binaryTree', adapterId: 'binary-search-tree', operations: bstOperations },
  planned('recursive-binary-search-tree', '搜索树', '递归二叉搜索树', 'RecursiveBinarySearchTree', 'binaryTree'),
  { id: 'avl-tree', category: '搜索树', label: 'AVL 树', implementation: 'AvlTree<i32>', status: 'available', viewKind: 'binaryTree', adapterId: 'avl-tree', operations: avlOperations },
  { id: 'red-black-tree', category: '搜索树', label: '红黑树', implementation: 'RedBlackTree<i32>（左倾红黑树）', status: 'available', viewKind: 'binaryTree', adapterId: 'red-black-tree', operations: redBlackOperations },
  {id:'binary-search-tree-map',category:'有序映射',label:'BST 映射',implementation:'BinarySearchTreeMap<i32, i32>',status:'available',viewKind:'treeMap',adapterId:'binary-search-tree-map',operations:treeMapOperations},
  planned('recursive-binary-search-tree-map', '有序映射', '递归 BST 映射', 'RecursiveBinarySearchTreeMap', 'binaryTree'),
  {id:'avl-tree-map',category:'有序映射',label:'AVL 映射',implementation:'AvlTreeMap<i32, i32>',status:'available',viewKind:'treeMap',adapterId:'avl-tree-map',operations:treeMapOperations},
  {id:'red-black-tree-map',category:'有序映射',label:'红黑树映射',implementation:'RedBlackTreeMap<i32, i32>',status:'available',viewKind:'treeMap',adapterId:'red-black-tree-map',operations:treeMapOperations},
  { id: 'binary-heap', category: '优先结构', label: '二叉堆', implementation: 'BinaryHeap<i32>（最大堆）', status: 'available', viewKind: 'heap', adapterId: 'binary-heap', operations: heapOperations },
  {id:'priority-queue',category:'优先结构',label:'优先队列',implementation:'PriorityQueue<i32, i32>',status:'available',viewKind:'priorityQueue',adapterId:'priority-queue',operations:priorityQueueOperations},
  { id: 'b-tree-map', category: '多路树', label: 'B 树映射', implementation: 'BTreeMap<i32, i32>（2–3–4 树）', status: 'available', viewKind: 'multiwayTree', adapterId: 'b-tree-map', operations: bTreeOperations },
  { id: 'b-tree-set', category: '多路树', label: 'B 树集合', implementation: 'BTreeSet<i32>（2–3–4 树）', status: 'available', viewKind: 'multiwayTree', adapterId: 'b-tree-set', operations: [{id:'insert',label:'插入元素',input:'integerKey'},{id:'search',label:'查找元素',input:'integerKey'},{id:'remove',label:'删除元素',input:'integerKey'}] },
  { id: 'b-plus-tree-map', category: '多路树', label: 'B+ 树映射', implementation: 'BPlusTreeMap<i32, i32>', status: 'available', viewKind: 'multiwayTree', adapterId: 'b-plus-tree-map', operations: bPlusMapOperations },
  { id: 'b-plus-tree-set', category: '多路树', label: 'B+ 树集合', implementation: 'BPlusTreeSet<i32>', status: 'available', viewKind: 'multiwayTree', adapterId: 'b-plus-tree-set', operations: bPlusSetOperations },
  {id:'hash-map',category:'哈希结构',label:'哈希映射',implementation:'HashMap<i32, i32>',status:'available',viewKind:'hashTable',adapterId:'hash-map',operations:hashMapOperations},
  {id:'hash-set',category:'哈希结构',label:'哈希集合',implementation:'HashSet<i32>',status:'available',viewKind:'hashTable',adapterId:'hash-set',operations:hashSetOperations},
  {id:'trie',category:'前缀结构',label:'Trie 前缀树',implementation:'Trie',status:'available',viewKind:'trie',adapterId:'trie',operations:trieOperations},
];

const adapterIds = new Set(visualizerAdapters.map((adapter) => adapter.id));
for (const lesson of lessonCatalog) {
  if (lesson.status === 'available' && (!lesson.adapterId || !adapterIds.has(lesson.adapterId))) {
    throw new Error(`Available lesson "${lesson.id}" has no registered adapter.`);
  }
}

export const availableLessons = lessonCatalog.filter((lesson) => lesson.status === 'available');

export function getLesson(id: string): LessonDescriptor | undefined {
  return lessonCatalog.find((lesson) => lesson.id === id);
}

export function getAvailableLesson(id: string): LessonDescriptor | undefined {
  const lesson = getLesson(id);
  return lesson?.status === 'available' ? lesson : undefined;
}

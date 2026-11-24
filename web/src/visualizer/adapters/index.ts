import type { VisualizerAdapter } from '../session';
import type { AvlTraceView, MultiwayTraceView, RedBlackTraceView, StructureView, VisualizerTrace } from '../../types';
import bstAdapter from './bst';
import vectorAdapter from './vector';
import singlyLinkedListAdapter from './singlyLinkedList';
import doublyLinkedListAdapter from './doublyLinkedList';
import heapAdapter from './heap';
import avlAdapter from './avl';
import redBlackAdapter from './redBlack';
import bTreeAdapter from './bTree';
import bTreeSetAdapter from './bTreeSet';
import bPlusAdapter from './bPlus';
import bPlusSetAdapter from './bPlusSet';
import hashMapAdapter from './hashMap';
import hashSetAdapter from './hashSet';
import { treeMapAdapters } from './treeMap';
import trieAdapter from './trie';
import { linearAdapters } from './linear';
import priorityQueueAdapter from './priorityQueue';

/** Add one adapter here when a structure is ready for the visualizer. */
export const visualizerAdapters: readonly VisualizerAdapter<StructureView, VisualizerTrace | AvlTraceView | RedBlackTraceView | MultiwayTraceView, string>[] = [bstAdapter, vectorAdapter, ...linearAdapters, singlyLinkedListAdapter, doublyLinkedListAdapter, heapAdapter, priorityQueueAdapter, avlAdapter, redBlackAdapter, bTreeAdapter, bTreeSetAdapter, bPlusAdapter, bPlusSetAdapter, hashMapAdapter, hashSetAdapter, ...treeMapAdapters, trieAdapter];

export function getVisualizerAdapter<TSnapshot = StructureView, TTrace = VisualizerTrace, TOperation extends string = string>(id: string): VisualizerAdapter<TSnapshot, TTrace, TOperation> {
  const adapter = visualizerAdapters.find((candidate) => candidate.id === id);
  if (!adapter) throw new Error(`Unknown data structure: ${id}`);
  return adapter as VisualizerAdapter<TSnapshot, TTrace, TOperation>;
}

import init, { DoublyLinkedListSession } from '../../generated/data_structures_visualizer_wasm';
import type { LinkedListTraceView, LinkedListView, VisualizerCommand } from '../../types';
import type { VisualizerAdapter, VisualizerSession } from '../session';

export type DoublyLinkedListOperation = 'pushFront' | 'pushBack' | 'popFront' | 'popBack' | 'front' | 'back';
const operations: readonly DoublyLinkedListOperation[] = ['pushFront', 'pushBack', 'popFront', 'popBack', 'front', 'back'];
let sessionPromise: Promise<DoublyLinkedListSession> | null = null;

const doublyLinkedListAdapter: VisualizerAdapter<LinkedListView, LinkedListTraceView, DoublyLinkedListOperation> = {
  id: 'doubly-linked-list',
  label: '双向链表',
  async initialize(): Promise<VisualizerSession<LinkedListView, LinkedListTraceView, DoublyLinkedListOperation>> {
    await init();
    sessionPromise ??= Promise.resolve().then(() => {
      const session = new DoublyLinkedListSession();
      session.load_example();
      return session;
    });
    return new DoublyLinkedListVisualizerSession(await sessionPromise);
  },
};

class DoublyLinkedListVisualizerSession implements VisualizerSession<LinkedListView, LinkedListTraceView, DoublyLinkedListOperation> {
  readonly structureId = doublyLinkedListAdapter.id;
  readonly operations = operations;

  constructor(private readonly session: DoublyLinkedListSession) {}

  snapshot(): LinkedListView {
    return this.session.snapshot();
  }

  loadExample(): LinkedListView {
    return this.session.load_example();
  }

  reset(): LinkedListView {
    return this.session.reset();
  }

  run(command: VisualizerCommand): LinkedListTraceView {
    if (command.structureId !== this.structureId) throw new Error('This command belongs to another lesson.');
    if (!operations.includes(command.operation as DoublyLinkedListOperation)) {
      throw new Error(`Unsupported doubly linked list operation: ${command.operation}`);
    }
    const operation = command.operation as DoublyLinkedListOperation;
    if (operation === 'pushFront' || operation === 'pushBack') {
      if (command.input.kind !== 'value') throw new Error('Adding a node expects an integer value.');
      const value = asI32(command.input.value);
      return operation === 'pushFront' ? this.session.push_front(value) : this.session.push_back(value);
    }
    if (command.input.kind !== 'none') throw new Error(`${operation} does not take input.`);
    switch (operation) {
      case 'popFront': return this.session.pop_front();
      case 'popBack': return this.session.pop_back();
      case 'front': return this.session.front();
      case 'back': return this.session.back();
    }
  }
}

function asI32(value: string | number): number {
  const number = Number(value);
  if (!Number.isInteger(number) || number < -2147483648 || number > 2147483647) {
    throw new Error('请输入 -2,147,483,648 到 2,147,483,647 之间的整数。');
  }
  return number;
}

export default doublyLinkedListAdapter;

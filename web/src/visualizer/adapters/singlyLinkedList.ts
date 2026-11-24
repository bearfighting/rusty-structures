import init, { SinglyLinkedListSession } from '../../generated/data_structures_visualizer_wasm';
import type { LinkedListTraceView, LinkedListView, VisualizerCommand } from '../../types';
import type { VisualizerAdapter, VisualizerSession } from '../session';

export type SinglyLinkedListOperation = 'pushFront' | 'pushBack' | 'popFront' | 'front' | 'back';
const operations: readonly SinglyLinkedListOperation[] = ['pushFront', 'pushBack', 'popFront', 'front', 'back'];
let sessionPromise: Promise<SinglyLinkedListSession> | null = null;

const singlyLinkedListAdapter: VisualizerAdapter<LinkedListView, LinkedListTraceView, SinglyLinkedListOperation> = {
  id: 'singly-linked-list',
  label: '单向链表',
  async initialize(): Promise<VisualizerSession<LinkedListView, LinkedListTraceView, SinglyLinkedListOperation>> {
    await init();
    sessionPromise ??= Promise.resolve().then(() => {
      const session = new SinglyLinkedListSession();
      session.load_example();
      return session;
    });
    return new SinglyLinkedListVisualizerSession(await sessionPromise);
  },
};

class SinglyLinkedListVisualizerSession implements VisualizerSession<LinkedListView, LinkedListTraceView, SinglyLinkedListOperation> {
  readonly structureId = singlyLinkedListAdapter.id;
  readonly operations = operations;

  constructor(private readonly session: SinglyLinkedListSession) {}

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
    if (!operations.includes(command.operation as SinglyLinkedListOperation)) {
      throw new Error(`Unsupported singly linked list operation: ${command.operation}`);
    }
    const operation = command.operation as SinglyLinkedListOperation;
    if (operation === 'pushFront' || operation === 'pushBack') {
      if (command.input.kind !== 'value') throw new Error('Adding a node expects an integer value.');
      const value = asI32(command.input.value);
      return operation === 'pushFront' ? this.session.push_front(value) : this.session.push_back(value);
    }
    if (command.input.kind !== 'none') throw new Error(`${operation} does not take input.`);
    switch (operation) {
      case 'popFront': return this.session.pop_front();
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

export default singlyLinkedListAdapter;

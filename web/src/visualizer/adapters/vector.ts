import init, { VectorSession } from '../../generated/data_structures_visualizer_wasm';
import type { SequenceTraceView, SequenceView, VisualizerCommand } from '../../types';
import type { VisualizerAdapter, VisualizerSession } from '../session';

export type VectorOperation = 'push' | 'pop' | 'insert' | 'remove' | 'get' | 'reserve';
const operations: readonly VectorOperation[] = ['push', 'pop', 'insert', 'remove', 'get', 'reserve'];
let sessionPromise: Promise<VectorSession> | null = null;

const vectorAdapter: VisualizerAdapter<SequenceView, SequenceTraceView, VectorOperation> = {
  id: 'vector',
  label: '动态数组',
  async initialize(): Promise<VisualizerSession<SequenceView, SequenceTraceView, VectorOperation>> {
    await init();
    sessionPromise ??= Promise.resolve().then(() => {
      const session = new VectorSession();
      session.load_example();
      return session;
    });
    return new VectorVisualizerSession(await sessionPromise);
  },
};

class VectorVisualizerSession implements VisualizerSession<SequenceView, SequenceTraceView, VectorOperation> {
  readonly structureId = vectorAdapter.id;
  readonly operations = operations;

  constructor(private readonly session: VectorSession) {}

  snapshot(): SequenceView {
    return this.session.snapshot();
  }

  loadExample(): SequenceView {
    return this.session.load_example();
  }

  reset(): SequenceView {
    return this.session.reset();
  }

  run(command: VisualizerCommand): SequenceTraceView {
    if (command.structureId !== this.structureId) throw new Error('This command belongs to another lesson.');
    if (!operations.includes(command.operation as VectorOperation)) {
      throw new Error(`Unsupported Vector operation: ${command.operation}`);
    }
    const operation = command.operation as VectorOperation;
    switch (operation) {
      case 'push':
        if (command.input.kind !== 'value') throw new Error('Append expects a value.');
        return this.session.push(asI32(command.input.value));
      case 'pop':
        if (command.input.kind !== 'none') throw new Error('Pop does not take input.');
        return this.session.pop();
      case 'insert':
        if (command.input.kind !== 'indexedValue') throw new Error('Insert expects an index and value.');
        return this.session.insert_at(asIndex(command.input.index), asI32(command.input.value));
      case 'remove':
        if (command.input.kind !== 'index') throw new Error('Remove expects an index.');
        return this.session.remove_at(asIndex(command.input.index));
      case 'get':
        if (command.input.kind !== 'index') throw new Error('Get expects an index.');
        return this.session.get(asIndex(command.input.index));
      case 'reserve':
        if (command.input.kind !== 'capacity') throw new Error('Reserve expects an additional capacity.');
        return this.session.reserve(asIndex(command.input.additional));
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

function asIndex(value: number): number {
  if (!Number.isSafeInteger(value) || value < 0 || value > 64) {
    throw new Error('索引/容量必须是 0 到 64 之间的整数。');
  }
  return value;
}

export default vectorAdapter;

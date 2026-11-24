import init, { HeapSession } from '../../generated/data_structures_visualizer_wasm';
import type { HeapTraceView, HeapView, VisualizerCommand } from '../../types';
import type { VisualizerAdapter, VisualizerSession } from '../session';

export type HeapOperation = 'push' | 'pop' | 'peek';
const operations: readonly HeapOperation[] = ['push', 'pop', 'peek'];
let sessionPromise: Promise<HeapSession> | null = null;

const heapAdapter: VisualizerAdapter<HeapView, HeapTraceView, HeapOperation> = {
  id: 'binary-heap', label: '二叉堆',
  async initialize() {
    await init();
    sessionPromise ??= Promise.resolve().then(() => { const session = new HeapSession(); session.load_example(); return session; });
    return new HeapVisualizerSession(await sessionPromise);
  },
};

class HeapVisualizerSession implements VisualizerSession<HeapView, HeapTraceView, HeapOperation> {
  readonly structureId = heapAdapter.id;
  readonly operations = operations;
  constructor(private readonly session: HeapSession) {}
  snapshot(): HeapView { return this.session.snapshot(); }
  loadExample(): HeapView { return this.session.load_example(); }
  reset(): HeapView { return this.session.reset(); }
  run(command: VisualizerCommand): HeapTraceView {
    if (command.structureId !== this.structureId) throw new Error('This command belongs to another lesson.');
    if (!operations.includes(command.operation as HeapOperation)) throw new Error(`Unsupported BinaryHeap operation: ${command.operation}`);
    if (command.operation === 'push') {
      if (command.input.kind !== 'value') throw new Error('Push expects an integer value.');
      const value = Number(command.input.value);
      if (!Number.isInteger(value) || value < -2147483648 || value > 2147483647) throw new Error('请输入有效的 32 位整数。');
      return this.session.push(value);
    }
    if (command.input.kind !== 'none') throw new Error('Pop and peek do not take input.');
    return command.operation === 'pop' ? this.session.pop() : this.session.peek();
  }
}

export default heapAdapter;

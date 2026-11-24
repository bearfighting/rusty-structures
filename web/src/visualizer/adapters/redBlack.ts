import init, { RedBlackSession } from '../../generated/data_structures_visualizer_wasm';
import type { RedBlackTraceView, TreeView, VisualizerCommand } from '../../types';
import type { VisualizerAdapter, VisualizerSession } from '../session';

export type RedBlackOperation = 'insert' | 'search' | 'remove';
const operations: readonly RedBlackOperation[] = ['insert', 'search', 'remove'];
let sessionPromise: Promise<RedBlackSession> | null = null;
const adapter: VisualizerAdapter<TreeView, RedBlackTraceView, RedBlackOperation> = {
  id: 'red-black-tree', label: '红黑树',
  async initialize() { await init(); sessionPromise ??= Promise.resolve().then(() => { const session = new RedBlackSession(); session.load_example(); return session; }); return new RedBlackVisualizerSession(await sessionPromise); },
};
class RedBlackVisualizerSession implements VisualizerSession<TreeView, RedBlackTraceView, RedBlackOperation> {
  readonly structureId = adapter.id; readonly operations = operations;
  constructor(private readonly session: RedBlackSession) {}
  snapshot(): TreeView { return this.session.snapshot(); }
  loadExample(): TreeView { return this.session.load_example(); }
  reset(): TreeView { return this.session.reset(); }
  run(command: VisualizerCommand): RedBlackTraceView {
    if (command.structureId !== this.structureId || command.input.kind !== 'integerKey') throw new Error('红黑树操作需要一个整数键。');
    if (!operations.includes(command.operation as RedBlackOperation)) throw new Error(`不支持的红黑树操作：${command.operation}`);
    if (command.operation === 'insert') return this.session.insert(command.input.value);
    return command.operation === 'search' ? this.session.search(command.input.value) : this.session.remove(command.input.value);
  }
}
export default adapter;

import init, { AvlSession } from '../../generated/data_structures_visualizer_wasm';
import type { AvlTraceView, TreeView, VisualizerCommand } from '../../types';
import type { VisualizerAdapter, VisualizerSession } from '../session';

export type AvlOperation = 'insert' | 'search' | 'remove';
const operations: readonly AvlOperation[] = ['insert', 'search', 'remove'];
let sessionPromise: Promise<AvlSession> | null = null;
const adapter: VisualizerAdapter<TreeView, AvlTraceView, AvlOperation> = {
  id: 'avl-tree', label: 'AVL 树',
  async initialize() { await init(); sessionPromise ??= Promise.resolve().then(() => { const session = new AvlSession(); session.load_example(); return session; }); return new AvlVisualizerSession(await sessionPromise); },
};
class AvlVisualizerSession implements VisualizerSession<TreeView, AvlTraceView, AvlOperation> {
  readonly structureId = adapter.id; readonly operations = operations;
  constructor(private readonly session: AvlSession) {}
  snapshot(): TreeView { return this.session.snapshot(); }
  loadExample(): TreeView { return this.session.load_example(); }
  reset(): TreeView { return this.session.reset(); }
  run(command: VisualizerCommand): AvlTraceView {
    if (command.structureId !== this.structureId || command.input.kind !== 'integerKey') throw new Error('AVL 操作需要一个整数键。');
    if (!operations.includes(command.operation as AvlOperation)) throw new Error(`不支持的 AVL 操作：${command.operation}`);
    if (command.operation === 'insert') return this.session.insert(command.input.value);
    return command.operation === 'search' ? this.session.search(command.input.value) : this.session.remove(command.input.value);
  }
}
export default adapter;

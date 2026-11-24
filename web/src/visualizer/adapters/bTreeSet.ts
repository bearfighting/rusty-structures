import init, { BTreeSetSession } from '../../generated/data_structures_visualizer_wasm';
import type { MultiwayTraceView, MultiwayTreeView, VisualizerCommand } from '../../types';
import type { VisualizerAdapter, VisualizerSession } from '../session';

export type BTreeSetOperation = 'insert' | 'search' | 'remove';
const operations: readonly BTreeSetOperation[] = ['insert', 'search', 'remove'];
let sessionPromise: Promise<BTreeSetSession> | null = null;
const adapter: VisualizerAdapter<MultiwayTreeView, MultiwayTraceView, BTreeSetOperation> = {
  id: 'b-tree-set', label: 'B 树集合',
  async initialize() { await init(); sessionPromise ??= Promise.resolve().then(() => { const session = new BTreeSetSession(); session.load_example(); return session; }); return new BTreeSetVisualizerSession(await sessionPromise); },
};
class BTreeSetVisualizerSession implements VisualizerSession<MultiwayTreeView, MultiwayTraceView, BTreeSetOperation> {
  readonly structureId = adapter.id; readonly operations = operations;
  constructor(private readonly session: BTreeSetSession) {}
  snapshot(): MultiwayTreeView { return this.session.snapshot(); }
  loadExample(): MultiwayTreeView { return this.session.load_example(); }
  reset(): MultiwayTreeView { return this.session.reset(); }
  run(command: VisualizerCommand): MultiwayTraceView {
    if (command.structureId !== this.structureId || command.input.kind !== 'integerKey') throw new Error('BTreeSet operation expects an integer.');
    const value = asI32(command.input.value);
    if (command.operation === 'insert') return this.session.insert(value);
    if (command.operation === 'search') return this.session.get(value);
    if (command.operation === 'remove') return this.session.remove(value);
    throw new Error(`不支持的 BTreeSet 操作：${command.operation}`);
  }
}
function asI32(v:number){if(!Number.isInteger(v)||v < -2147483648||v>2147483647)throw new Error('请输入有效的 32 位整数。');return v;}
export default adapter;

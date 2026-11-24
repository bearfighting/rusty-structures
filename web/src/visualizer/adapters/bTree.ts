import init, { BTreeSession } from '../../generated/data_structures_visualizer_wasm';
import type { MultiwayTraceView, MultiwayTreeView, VisualizerCommand } from '../../types';
import type { VisualizerAdapter, VisualizerSession } from '../session';

export type BTreeOperation = 'insert' | 'search' | 'remove';
const operations: readonly BTreeOperation[] = ['insert', 'search', 'remove'];
let sessionPromise: Promise<BTreeSession> | null = null;
const adapter: VisualizerAdapter<MultiwayTreeView, MultiwayTraceView, BTreeOperation> = {
  id: 'b-tree-map', label: 'B 树映射',
  async initialize() { await init(); sessionPromise ??= Promise.resolve().then(() => { const session = new BTreeSession(); session.load_example(); return session; }); return new BTreeVisualizerSession(await sessionPromise); },
};
class BTreeVisualizerSession implements VisualizerSession<MultiwayTreeView, MultiwayTraceView, BTreeOperation> {
  readonly structureId = adapter.id; readonly operations = operations;
  constructor(private readonly session: BTreeSession) {}
  snapshot(): MultiwayTreeView { return this.session.snapshot(); }
  loadExample(): MultiwayTreeView { return this.session.load_example(); }
  reset(): MultiwayTreeView { return this.session.reset(); }
  run(command: VisualizerCommand): MultiwayTraceView {
    if (command.structureId !== this.structureId) throw new Error('This command belongs to another lesson.');
    if (command.operation === 'insert') {
      if (command.input.kind !== 'keyValue') throw new Error('BTreeMap insertion expects a key and value.');
      return this.session.insert(asI32(command.input.key), asI32(command.input.value));
    }
    if (command.input.kind !== 'integerKey') throw new Error('BTreeMap lookup/removal expects an integer key.');
    if (command.operation === 'search') return this.session.get(asI32(command.input.value));
    if (command.operation === 'remove') return this.session.remove(asI32(command.input.value));
    throw new Error(`不支持的 BTreeMap 操作：${command.operation}`);
  }
}
function asI32(value: string | number) { const n = Number(value); if (!Number.isInteger(n) || n < -2147483648 || n > 2147483647) throw new Error('请输入有效的 32 位整数。'); return n; }
export default adapter;

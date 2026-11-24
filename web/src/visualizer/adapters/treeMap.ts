import init, { TreeMapSession } from '../../generated/data_structures_visualizer_wasm';
import type { TreeMapTraceView, TreeMapView, VisualizerCommand } from '../../types';
import type { VisualizerAdapter, VisualizerSession } from '../session';

export type TreeMapOperation = 'insert' | 'search' | 'remove' | 'scan';
const operations: readonly TreeMapOperation[] = ['insert', 'search', 'remove', 'scan'];
const labels: Record<string, string> = {
  'binary-search-tree-map': 'BST 映射',
  'avl-tree-map': 'AVL 映射',
  'red-black-tree-map': '红黑树映射',
};

function adapter(id: keyof typeof labels): VisualizerAdapter<TreeMapView, TreeMapTraceView, TreeMapOperation> {
  let promise: Promise<TreeMapSession> | null = null;
  return {
    id,
    label: labels[id],
    async initialize() {
      await init();
      promise ??= Promise.resolve().then(() => {
        const session = new TreeMapSession(id);
        session.load_example();
        return session;
      });
      return new Session(id, await promise);
    },
  };
}

class Session implements VisualizerSession<TreeMapView, TreeMapTraceView, TreeMapOperation> {
  readonly operations = operations;
  constructor(readonly structureId: string, private readonly session: TreeMapSession) {}
  snapshot() { return this.session.snapshot(); }
  loadExample() { return this.session.load_example(); }
  reset() { return this.session.reset(); }
  run(command: VisualizerCommand) {
    if (command.structureId !== this.structureId) throw new Error('Command belongs to another lesson.');
    if (command.operation === 'scan') return this.session.scan();
    if (command.operation === 'insert') {
      if (command.input.kind !== 'keyValue') throw new Error('Insert expects a key and value.');
      return this.session.insert(int32(command.input.key), int32(command.input.value));
    }
    if (command.input.kind !== 'integerKey') throw new Error('Expected an integer key.');
    return command.operation === 'search' ? this.session.get(int32(command.input.value)) : this.session.remove(int32(command.input.value));
  }
}

function int32(value: number | string) {
  const number = Number(value);
  if (!Number.isInteger(number) || number < -2147483648 || number > 2147483647) throw new Error('请输入有效的 32 位整数。');
  return number;
}

export const treeMapAdapters = [
  adapter('binary-search-tree-map'),
  adapter('avl-tree-map'),
  adapter('red-black-tree-map'),
] as const;

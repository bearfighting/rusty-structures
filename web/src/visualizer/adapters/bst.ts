import init, { BstSession } from '../../generated/data_structures_visualizer_wasm';
import type { TraceView, TreeView, VisualizerCommand } from '../../types';
import type { Operation, VisualizerAdapter, VisualizerSession } from '../session';

let sessionPromise: Promise<BstSession> | null = null;

const bstAdapter: VisualizerAdapter = {
  id: 'binary-search-tree',
  label: '二叉搜索树',
  async initialize(): Promise<VisualizerSession<TreeView, TraceView>> {
    await init();
    sessionPromise ??= Promise.resolve().then(() => {
      const session = new BstSession();
      session.load_example();
      return session;
    });
    return new BstVisualizerSession(await sessionPromise);
  },
};

class BstVisualizerSession implements VisualizerSession {
  readonly structureId = bstAdapter.id;
  readonly operations = ['insert', 'search', 'remove'] as const;

  constructor(private readonly session: BstSession) {}

  snapshot(): TreeView {
    return this.session.snapshot();
  }

  loadExample(): TreeView {
    return this.session.load_example();
  }

  reset(): TreeView {
    return this.session.reset();
  }

  run(command: VisualizerCommand): TraceView {
    if (command.structureId !== this.structureId || command.input.kind !== 'integerKey') {
      throw new Error('This lesson expects an integer-key command.');
    }
    const operation: Operation = command.operation as Operation;
    if (!this.operations.includes(operation)) throw new Error(`Unsupported BST operation: ${command.operation}`);
    return this.session[operation](command.input.value);
  }
}

export default bstAdapter;

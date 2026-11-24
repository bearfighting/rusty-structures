import type { StructureView, TraceView, TreeView, VisualizerCommand } from '../types';

export type Operation = 'insert' | 'search' | 'remove';

/** A structure-specific bridge between its view model and a Rust/WASM implementation. */
export interface VisualizerSession<TSnapshot = TreeView, TTrace = TraceView, TOperation extends string = string> {
  readonly structureId: string;
  readonly operations: readonly TOperation[];
  snapshot(): TSnapshot;
  loadExample(): TSnapshot;
  reset(): TSnapshot;
  run(command: VisualizerCommand): TTrace;
}

export interface VisualizerAdapter<TSnapshot = StructureView, TTrace = TraceView, TOperation extends string = string> {
  readonly id: string;
  readonly label: string;
  initialize(): Promise<VisualizerSession<TSnapshot, TTrace, TOperation>>;
}

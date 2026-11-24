import type { TraceEvent, TraceView } from '../types';
import type { Operation } from './session';

export function eventTitle(event: TraceEvent | null, trace: TraceView | null, cursor: number) {
  if (!trace) return '准备开始';
  if (!trace.events.length) return '操作已完成';
  if (cursor >= trace.events.length) return '操作已完成';
  switch (event?.kind) {
    case 'compare': return `比较 ${event.key} 和 ${event.nodeValue}`;
    case 'descend': return event.direction === 'left' ? '向左子树继续' : '向右子树继续';
    case 'inserted': return `插入节点 ${event.key}`;
    case 'duplicate': return `节点 ${event.key} 已存在`;
    case 'found': return `找到节点 ${event.key}`;
    case 'missing': return `没有找到 ${event.key}`;
    case 'successorSelected': return `选择后继节点 ${event.successor}`;
    case 'removed': return `移除节点 ${event.key}`;
    default: return '操作已完成';
  }
}

export function eventExplanation(event: TraceEvent | null, trace: TraceView | null, cursor: number) {
  if (!trace) return '选择一个操作并输入整数。每一步都会由 Rust 中的二叉搜索树实际执行。';
  if (!trace.events.length || cursor >= trace.events.length) return `操作结束。当前树包含 ${trace.after.len} 个节点，画布显示 Rust 返回的最终结构。`;
  switch (event?.kind) {
    case 'compare':
      return event.ordering === 'less' ? `${event.key} 小于当前节点 ${event.nodeValue}，按照 BST 规则转向左子树。` : event.ordering === 'greater' ? `${event.key} 大于当前节点 ${event.nodeValue}，按照 BST 规则转向右子树。` : `键 ${event.key} 与当前节点相等。查找已命中；插入会报告重复键。`;
    case 'descend': return `从节点 ${event.fromId} 沿${event.direction === 'left' ? '左' : '右'}链接继续查找。`;
    case 'inserted': return `目标链接为空，Rust 实现把 ${event.key} 放在这里，树大小增加一。`;
    case 'duplicate': return `树中已经有 ${event.key}。集合语义不插入重复值，结构保持不变。`;
    case 'found': return `找到 ${event.key}，查找路径到此结束。`;
    case 'missing': return `走到空链接仍未命中，${event.key} 不在树中。`;
    case 'successorSelected': return `节点 ${event.removed} 有两个孩子；先选右子树中最小的节点 ${event.successor} 作为中序后继。`;
    case 'removed': return `节点 ${event.key} 已移除，剩余子树接回原链接。`;
    default: return '观察当前操作路径。';
  }
}

export function shortEvent(event: TraceEvent) {
  switch (event.kind) {
    case 'compare': return `比较 ${event.key} 与 ${event.nodeValue}`;
    case 'descend': return `进入${event.direction === 'left' ? '左' : '右'}子树`;
    case 'inserted': return `插入 ${event.key}`;
    case 'duplicate': return `${event.key} 已存在`;
    case 'found': return `找到 ${event.key}`;
    case 'missing': return `未找到 ${event.key}`;
    case 'successorSelected': return `后继：${event.successor}`;
    case 'removed': return `移除 ${event.key}`;
  }
}

export function pseudoLines(operation: Operation) {
  if (operation === 'search') return ['current = root', '比较 key 与 current.value', '若 key 更小，进入 left', '若 key 更大，进入 right', '相等则返回 found', '空链接则返回 missing'];
  if (operation === 'remove') return ['current = root', '比较 key 与 current.value', '按大小继续向下查找', '0 或 1 个孩子：接回子树', '2 个孩子：寻找中序后继', '移除节点并维护链接'];
  return ['current = root', '比较 key 与 current.value', '若 key 更小，进入 left', '若 key 更大，进入 right', '空链接处创建新节点', '相等则保持树不变'];
}

export function pseudoLine(event: TraceEvent | null) {
  if (!event) return -1;
  if (event.kind === 'compare') return 1;
  if (event.kind === 'descend') return event.direction === 'left' ? 2 : 3;
  if (event.kind === 'inserted') return 4;
  if (event.kind === 'duplicate') return 5;
  if (event.kind === 'successorSelected') return 4;
  if (event.kind === 'removed') return 5;
  if (event.kind === 'found') return 4;
  return 5;
}


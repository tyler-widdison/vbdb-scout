import type { Ref, InjectionKey } from "vue"

export type NodeType = "association" | "season" | "match"

export interface NodeData {
  id: number
  name: string
  type: NodeType
  parentId?: number
  children: NodeData[]
  expanded: boolean
  loaded: boolean
}

export interface TreeContext {
  selectedKey: Ref<string | null>
  focused: Ref<boolean>
  renamingKey: Ref<string | null>
  select: (key: string) => void
  rename: (key: string) => void
  focus: () => void
  delete: (key: string) => Promise<void>
  open: (key: string) => Promise<void>
  stopRename: () => void
}

export const TREE_INJECTION: InjectionKey<TreeContext> = Symbol("tree")

export function nodeKey(node: NodeData): string {
  return `${node.type}-${node.id}`
}

export function getVisibleNodes(roots: NodeData[]): NodeData[] {
  const result: NodeData[] = []
  for (const node of roots) {
    result.push(node)
    if (node.expanded) result.push(...getVisibleNodes(node.children))
  }
  return result
}

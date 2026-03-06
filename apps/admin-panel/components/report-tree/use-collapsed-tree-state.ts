import { useCallback, useEffect, useState } from "react"

interface TreeNode<T> {
  children?: T[]
}

interface UseCollapsedTreeStateResult {
  collapsedNodeIds: Set<string>
  toggleCollapsedNode: (nodeId: string) => void
}

export function useCollapsedTreeState<T extends TreeNode<T>>(
  nodes: T[],
  getNodeId: (node: T) => string,
): UseCollapsedTreeStateResult {
  const [collapsedNodeIds, setCollapsedNodeIds] = useState<Set<string>>(new Set())

  const toggleCollapsedNode = useCallback((nodeId: string) => {
    setCollapsedNodeIds((previous) => {
      const next = new Set(previous)
      if (next.has(nodeId)) {
        next.delete(nodeId)
      } else {
        next.add(nodeId)
      }
      return next
    })
  }, [])

  useEffect(() => {
    setCollapsedNodeIds(collectExpandableNodeIds(nodes, getNodeId))
  }, [getNodeId, nodes])

  return { collapsedNodeIds, toggleCollapsedNode }
}

function collectExpandableNodeIds<T extends TreeNode<T>>(
  nodes: T[],
  getNodeId: (node: T) => string,
): Set<string> {
  const expandableNodeIds = new Set<string>()

  const walk = (node: T) => {
    if ((node.children?.length ?? 0) > 0) {
      expandableNodeIds.add(getNodeId(node))
      node.children?.forEach(walk)
    }
  }

  nodes.forEach(walk)
  return expandableNodeIds
}

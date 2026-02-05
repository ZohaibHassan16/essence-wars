<script lang="ts">
  import { SvelteSet } from "svelte/reactivity";
  import type { TreeNodeDto } from "$lib/api/types";
  import TreeNodeView from "./TreeNodeView.svelte";

  let {
    treeRoot,
    algorithm = "Search",
    totalSimulations,
  }: {
    treeRoot: TreeNodeDto | null;
    algorithm?: string;
    totalSimulations?: number;
  } = $props();

  // Track expanded nodes
  const expandedNodes = new SvelteSet<string>(["root"]);

  // Expand/collapse all
  function expandAll() {
    if (!treeRoot) return;

    function addChildren(node: TreeNodeDto) {
      for (const child of node.children) {
        const childId = `${child.depth}-${child.actionStr.replace(/\s+/g, "-")}`;
        expandedNodes.add(childId);
        if (child.children.length > 0) {
          addChildren(child);
        }
      }
    }

    addChildren(treeRoot);
  }

  function collapseAll() {
    expandedNodes.clear();
    expandedNodes.add("root");
  }

  // Count visible nodes
  const visibleNodeCount = $derived(() => {
    if (!treeRoot) return 0;
    let count = 1; // root

    function countChildren(node: TreeNodeDto, nodeId: string) {
      if (!expandedNodes.has(nodeId)) return;
      for (const child of node.children) {
        count++;
        const childId = `${child.depth}-${child.actionStr.replace(/\s+/g, "-")}`;
        if (child.children.length > 0) {
          countChildren(child, childId);
        }
      }
    }

    countChildren(treeRoot, "root");
    return count;
  });
</script>

<div class="search-tree-view">
  <!-- Header -->
  <div class="tree-header">
    <div class="header-left">
      <span class="tree-icon">&#128065;</span>
      <span class="tree-title">Search Tree</span>
      {#if algorithm}
        <span class="algorithm-badge">{algorithm}</span>
      {/if}
    </div>
    <div class="header-right">
      {#if totalSimulations}
        <span class="stats">{totalSimulations.toLocaleString()} sims</span>
      {/if}
      <button class="control-btn" onclick={expandAll} title="Expand All">
        +
      </button>
      <button class="control-btn" onclick={collapseAll} title="Collapse All">
        -
      </button>
    </div>
  </div>

  <!-- Tree content -->
  <div class="tree-content">
    {#if treeRoot}
      <TreeNodeView
        node={treeRoot}
        isRoot={true}
        {expandedNodes}
      />
    {:else}
      <div class="no-tree">
        No search tree data available.
      </div>
    {/if}
  </div>

  <!-- Footer with legend -->
  <div class="tree-footer">
    <div class="legend">
      <span class="legend-item">
        <span class="legend-color best"></span>
        Best Path
      </span>
      <span class="legend-item">
        <span class="legend-color other"></span>
        Other
      </span>
    </div>
    <span class="node-count">{visibleNodeCount()} nodes</span>
  </div>
</div>

<style>
  .search-tree-view {
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 0.5rem;
    overflow: hidden;
  }

  .tree-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 0.625rem;
    background: rgba(0, 0, 0, 0.2);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .tree-icon {
    font-size: 0.875rem;
  }

  .tree-title {
    font-size: 0.75rem;
    font-weight: 600;
    color: #e0e0e0;
  }

  .algorithm-badge {
    font-size: 0.625rem;
    padding: 0.125rem 0.375rem;
    background: rgba(59, 130, 246, 0.2);
    color: #60a5fa;
    border-radius: 0.25rem;
    font-weight: 500;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 0.375rem;
  }

  .stats {
    font-size: 0.65rem;
    color: #9ca3af;
  }

  .control-btn {
    width: 1.25rem;
    height: 1.25rem;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.75rem;
    font-weight: bold;
    color: #9ca3af;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 0.25rem;
    cursor: pointer;
    transition: all 0.1s;
  }

  .control-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #e0e0e0;
  }

  .tree-content {
    padding: 0.5rem;
    max-height: 200px;
    overflow-y: auto;
  }

  .no-tree {
    padding: 1rem;
    text-align: center;
    color: #6b7280;
    font-size: 0.75rem;
  }

  .tree-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.375rem 0.625rem;
    background: rgba(0, 0, 0, 0.2);
    border-top: 1px solid rgba(255, 255, 255, 0.1);
  }

  .legend {
    display: flex;
    gap: 0.75rem;
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.6rem;
    color: #9ca3af;
  }

  .legend-color {
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 2px;
  }

  .legend-color.best {
    background: #34d399;
  }

  .legend-color.other {
    background: #6b7280;
  }

  .node-count {
    font-size: 0.6rem;
    color: #6b7280;
  }
</style>

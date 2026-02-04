<script lang="ts">
  import type { TreeNodeDto } from "$lib/api/types";

  let {
    node,
    isRoot = false,
    expandedNodes = $bindable(new Set<string>()),
    onNodeClick,
  }: {
    node: TreeNodeDto;
    isRoot?: boolean;
    expandedNodes?: Set<string>;
    onNodeClick?: (node: TreeNodeDto) => void;
  } = $props();

  // Generate a unique ID for this node
  const nodeId = $derived(
    isRoot ? "root" : `${node.depth}-${node.actionStr.replace(/\s+/g, "-")}`
  );

  const isExpanded = $derived(expandedNodes.has(nodeId));
  const hasChildren = $derived(node.children.length > 0);

  function toggleExpand() {
    if (!hasChildren) return;

    if (expandedNodes.has(nodeId)) {
      expandedNodes.delete(nodeId);
    } else {
      expandedNodes.add(nodeId);
    }
    expandedNodes = new Set(expandedNodes);
  }

  function handleClick() {
    if (onNodeClick) {
      onNodeClick(node);
    }
  }

  // Color based on score
  function getScoreColor(score: number): string {
    if (score >= 0.6) return "text-green-400";
    if (score >= 0.45) return "text-yellow-400";
    return "text-red-400";
  }

  // Bar width for visits visualization
  function getVisitBarWidth(visits: number, maxVisits: number): number {
    if (maxVisits === 0) return 0;
    return Math.max(5, (visits / maxVisits) * 100);
  }

  // Find max visits among siblings for bar scaling
  const maxSiblingVisits = $derived(() => {
    if (isRoot) return node.visits;
    return Math.max(...node.children.map((c) => c.visits), 1);
  });
</script>

<div class="tree-node" class:is-root={isRoot} class:is-best-path={node.isBestPath}>
  <!-- Node header -->
  <button
    class="node-header"
    class:expandable={hasChildren}
    onclick={toggleExpand}
    ondblclick={handleClick}
  >
    <!-- Expand/collapse indicator -->
    {#if hasChildren}
      <span class="expand-icon">{isExpanded ? "▼" : "▶"}</span>
    {:else}
      <span class="expand-icon-placeholder"></span>
    {/if}

    <!-- Node content -->
    <div class="node-content">
      <!-- Action name -->
      <span class="action-name" class:root-label={isRoot}>
        {isRoot ? "Position" : node.actionStr}
      </span>

      <!-- Stats -->
      <div class="node-stats">
        {#if node.visits > 0}
          <span class="visits" title="Visits">{node.visits.toLocaleString()}</span>
        {/if}
        <span class={`score ${getScoreColor(node.score)}`} title="Win Rate">
          {node.scoreDisplay}
        </span>
        {#if node.isBestPath && !isRoot}
          <span class="best-badge" title="Chosen Move">*</span>
        {/if}
      </div>
    </div>

    <!-- Visit bar (visual indicator) -->
    {#if !isRoot && node.visits > 0}
      <div class="visit-bar-container">
        <div
          class="visit-bar"
          class:best-path={node.isBestPath}
          style="width: {getVisitBarWidth(node.visits, maxSiblingVisits())}%"
        ></div>
      </div>
    {/if}
  </button>

  <!-- Children (if expanded) -->
  {#if isExpanded && hasChildren}
    <div class="children">
      {#each node.children as child, i (i)}
        <svelte:self
          node={child}
          {expandedNodes}
          {onNodeClick}
        />
      {/each}

      <!-- Truncation indicator -->
      {#if node.isTruncated}
        <div class="truncated-indicator">
          +{node.truncatedChildCount} more moves...
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .tree-node {
    font-size: 0.75rem;
    font-family: ui-monospace, monospace;
  }

  .node-header {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    width: 100%;
    padding: 0.25rem 0.375rem;
    border-radius: 0.25rem;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid transparent;
    cursor: default;
    text-align: left;
    transition: background-color 0.1s;
  }

  .node-header.expandable {
    cursor: pointer;
  }

  .node-header:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .is-best-path > .node-header {
    background: rgba(52, 211, 153, 0.08);
    border-color: rgba(52, 211, 153, 0.2);
  }

  .is-root > .node-header {
    background: rgba(59, 130, 246, 0.1);
    border-color: rgba(59, 130, 246, 0.3);
  }

  .expand-icon {
    width: 0.75rem;
    color: rgba(255, 255, 255, 0.4);
    font-size: 0.6rem;
    flex-shrink: 0;
  }

  .expand-icon-placeholder {
    width: 0.75rem;
    flex-shrink: 0;
  }

  .node-content {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    min-width: 0;
  }

  .action-name {
    color: #d1d5db;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 120px;
  }

  .root-label {
    color: #60a5fa;
    font-weight: 600;
  }

  .node-stats {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
  }

  .visits {
    color: #6b7280;
    font-size: 0.65rem;
  }

  .score {
    font-weight: 600;
    min-width: 2.5rem;
    text-align: right;
  }

  .best-badge {
    color: #fbbf24;
    font-weight: bold;
  }

  .visit-bar-container {
    width: 40px;
    height: 4px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 2px;
    overflow: hidden;
    flex-shrink: 0;
  }

  .visit-bar {
    height: 100%;
    background: #6b7280;
    border-radius: 2px;
    transition: width 0.2s;
  }

  .visit-bar.best-path {
    background: #34d399;
  }

  .children {
    margin-left: 1rem;
    padding-left: 0.5rem;
    border-left: 1px solid rgba(255, 255, 255, 0.1);
    margin-top: 0.125rem;
  }

  .truncated-indicator {
    padding: 0.25rem 0.375rem;
    color: #6b7280;
    font-size: 0.65rem;
    font-style: italic;
  }
</style>

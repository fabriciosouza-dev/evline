<script lang="ts">
  interface Tab {
    id: string;
    title: string;
  }

  let {
    tabs,
    activeTabId,
    onSelect,
    onClose,
    onNew,
    onRename,
    onReorder,
  }: {
    tabs: Tab[];
    activeTabId: string;
    onSelect: (id: string) => void;
    onClose: (id: string) => void;
    onNew: () => void;
    onRename?: (id: string, title: string) => void;
    onReorder?: (fromIndex: number, toIndex: number) => void;
  } = $props();

  let editingTabId = $state<string | null>(null);
  let editValue = $state("");
  let inputEl: HTMLInputElement | null = $state(null);
  let draggedIndex = $state<number | null>(null);
  let dragOverIndex = $state<number | null>(null);

  function startEditing(tabId: string, currentTitle: string) {
    editingTabId = tabId;
    editValue = currentTitle;
    setTimeout(() => inputEl?.select(), 0);
  }

  function finishEditing() {
    if (editingTabId && onRename) {
      onRename(editingTabId, editValue.trim());
    }
    editingTabId = null;
  }

  function onEditKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      finishEditing();
    } else if (e.key === "Escape") {
      editingTabId = null;
    }
  }

  function onDragStart(e: DragEvent, index: number) {
    draggedIndex = index;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = "move";
      e.dataTransfer.setData("text/plain", String(index));
    }
  }

  function onDragOver(e: DragEvent, index: number) {
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
    dragOverIndex = index;
  }

  function onDragLeave() {
    dragOverIndex = null;
  }

  function onDrop(e: DragEvent, toIndex: number) {
    e.preventDefault();
    if (draggedIndex !== null && draggedIndex !== toIndex && onReorder) {
      onReorder(draggedIndex, toIndex);
    }
    draggedIndex = null;
    dragOverIndex = null;
  }

  function onDragEnd() {
    draggedIndex = null;
    dragOverIndex = null;
  }
</script>

<div class="tab-bar">
  <div class="tabs-container">
    {#each tabs as tab, i (tab.id)}
      <div
        class="tab"
        class:active={tab.id === activeTabId}
        class:dragging={draggedIndex === i}
        class:drag-over={dragOverIndex === i && draggedIndex !== i}
        role="tab"
        tabindex="0"
        aria-selected={tab.id === activeTabId}
        draggable={editingTabId !== tab.id}
        onclick={() => onSelect(tab.id)}
        ondblclick={() => startEditing(tab.id, tab.title)}
        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') onSelect(tab.id); }}
        ondragstart={(e) => onDragStart(e, i)}
        ondragover={(e) => onDragOver(e, i)}
        ondragleave={onDragLeave}
        ondrop={(e) => onDrop(e, i)}
        ondragend={onDragEnd}
      >
        {#if editingTabId === tab.id}
          <input
            class="tab-edit-input"
            bind:this={inputEl}
            bind:value={editValue}
            onblur={finishEditing}
            onkeydown={onEditKeydown}
            onclick={(e) => e.stopPropagation()}
          />
        {:else}
          <span class="tab-title">{tab.title}</span>
        {/if}
        {#if tabs.length > 1 && editingTabId !== tab.id}
          <button
            class="tab-close"
            onclick={(e) => { e.stopPropagation(); onClose(tab.id); }}
            aria-label="Close tab"
          >&times;</button>
        {/if}
      </div>
    {/each}
  </div>
  <button class="tab-new" onclick={onNew} title="New tab">+</button>
</div>

<style>
  .tab-bar {
    display: flex;
    align-items: center;
    background: var(--bg-deep);
    border-bottom: 1px solid var(--border);
    padding: 0 4px;
    height: 36px;
    gap: 2px;
    user-select: none;
    -webkit-app-region: drag;
  }

  .tabs-container {
    display: flex;
    align-items: center;
    gap: 2px;
    overflow-x: auto;
    flex: 1;
    -webkit-app-region: no-drag;
  }

  .tabs-container::-webkit-scrollbar {
    display: none;
  }

  .tab {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 12px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    font-size: 12px;
    font-family: inherit;
    cursor: grab;
    border-radius: 6px 6px 0 0;
    white-space: nowrap;
    max-width: 160px;
    transition: all 0.15s;
    position: relative;
    outline: none;
  }

  .tab:hover {
    background: var(--bg-surface);
    color: var(--text-secondary);
  }

  .tab.active {
    background: var(--bg-surface);
    color: var(--text-primary);
    font-weight: 500;
  }

  .tab.active::after {
    content: "";
    position: absolute;
    bottom: 0;
    left: 8px;
    right: 8px;
    height: 2px;
    background: var(--accent);
    border-radius: 2px 2px 0 0;
  }

  .tab.dragging {
    opacity: 0.4;
  }

  .tab.drag-over {
    border-left: 2px solid var(--accent);
  }

  .tab-title {
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tab-edit-input {
    background: var(--bg-elevated);
    border: 1px solid var(--border-strong);
    border-radius: 3px;
    color: var(--text-primary);
    font-size: 12px;
    font-family: inherit;
    padding: 1px 4px;
    width: 80px;
    outline: none;
  }

  .tab-edit-input:focus {
    border-color: var(--accent);
  }

  .tab-close {
    font-size: 15px;
    line-height: 1;
    opacity: 0;
    color: var(--text-muted);
    border-radius: 3px;
    padding: 0 2px;
    transition: all 0.1s;
    background: none;
    border: none;
    cursor: pointer;
    font-family: inherit;
  }

  .tab:hover .tab-close {
    opacity: 1;
  }

  .tab-close:hover {
    color: var(--red);
    background: var(--bg-elevated);
  }

  .tab-new {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    font-size: 18px;
    cursor: pointer;
    border-radius: 6px;
    transition: all 0.15s;
    flex-shrink: 0;
    -webkit-app-region: no-drag;
  }

  .tab-new:hover {
    background: var(--bg-elevated);
    color: var(--green);
  }
</style>

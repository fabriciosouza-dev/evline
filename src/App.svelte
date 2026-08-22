<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import TabBar from "./lib/TabBar.svelte";
  import EditorPane from "./lib/EditorPane.svelte";
  import { t, type UiLocale } from "./lib/i18n";
  import { applyTheme, type Theme } from "./lib/themes";

  interface Tab {
    id: string;
    title: string;
    text: string;
  }

  interface SavedState {
    tabs: { id: string; title: string; text: string }[];
    active_tab_id: string;
    ui_locale: string;
    tab_counter: number;
    theme?: string;
  }

  let tabCounter = $state(1);
  let tabs: Tab[] = $state([{ id: "tab-1", title: "", text: "" }]);
  let activeTabId = $state("tab-1");
  let showHelp = $state(false);
  let showSettings = $state(false);
  let uiLocale: UiLocale = $state("pt-br");
  let theme: Theme = $state("dark");
  let tabTotals: Record<string, string> = $state({});
  let loaded = $state(false);
  let saveTimer: any;
  let closedTabs: Tab[] = $state([]);
  let copiedTotal = $state(false);

  let strings = $derived(t(uiLocale));
  let activeTab = $derived(tabs.find(t => t.id === activeTabId)!);
  let activeTotalFormatted = $derived(tabTotals[activeTabId] || "");

  let displayTabs = $derived(tabs.map((tab, i) => ({
    id: tab.id,
    title: tab.title || `${strings.tabDefault} ${i + 1}`,
  })));

  function scheduleSave() {
    if (!loaded) return;
    clearTimeout(saveTimer);
    saveTimer = setTimeout(persistState, 500);
  }

  async function persistState() {
    const data: SavedState = {
      tabs: tabs.map(tab => ({ id: tab.id, title: tab.title, text: tab.text })),
      active_tab_id: activeTabId,
      ui_locale: uiLocale,
      tab_counter: tabCounter,
      theme,
    };
    try {
      await invoke("save_state", { data });
    } catch (e) {
      console.warn("Failed to save state:", e);
    }
  }

  async function loadPersistedState() {
    try {
      const saved = await invoke<SavedState | null>("load_state");
      if (saved && saved.tabs && saved.tabs.length > 0) {
        tabs = saved.tabs;
        activeTabId = saved.active_tab_id || saved.tabs[0].id;
        uiLocale = (saved.ui_locale === "en" ? "en" : "pt-br") as UiLocale;
        tabCounter = saved.tab_counter || saved.tabs.length;
        if (saved.theme === "light" || saved.theme === "dark") {
          theme = saved.theme as Theme;
        }
      }
    } catch (e) {
      console.warn("Failed to load state:", e);
    }
    applyTheme(theme);
    loaded = true;
  }

  function createTab() {
    tabCounter++;
    const newTab: Tab = {
      id: `tab-${tabCounter}`,
      title: "",
      text: "",
    };
    tabs = [...tabs, newTab];
    activeTabId = newTab.id;
    scheduleSave();
  }

  function closeTab(id: string) {
    if (tabs.length <= 1) return;

    const idx = tabs.findIndex(t => t.id === id);
    const closedTab = tabs[idx];

    closedTabs = [...closedTabs, { ...closedTab }].slice(-10);

    invoke("close_tab", { tabId: id }).catch(() => {});

    tabs = tabs.filter(t => t.id !== id);

    if (activeTabId === id) {
      const newIdx = Math.min(idx, tabs.length - 1);
      activeTabId = tabs[newIdx].id;
    }
    scheduleSave();
  }

  function reopenTab() {
    if (closedTabs.length === 0) return;
    const tab = closedTabs[closedTabs.length - 1];
    closedTabs = closedTabs.slice(0, -1);
    tabs = [...tabs, tab];
    activeTabId = tab.id;
    scheduleSave();
  }

  function selectTab(id: string) {
    activeTabId = id;
    scheduleSave();
  }

  function renameTab(id: string, newTitle: string) {
    const tab = tabs.find(t => t.id === id);
    if (tab) {
      tab.title = newTitle;
      tabs = tabs;
      scheduleSave();
    }
  }

  function reorderTabs(fromIndex: number, toIndex: number) {
    const updated = [...tabs];
    const [moved] = updated.splice(fromIndex, 1);
    updated.splice(toIndex, 0, moved);
    tabs = updated;
    scheduleSave();
  }

  function onTextChange(tabId: string, text: string) {
    const tab = tabs.find(t => t.id === tabId);
    if (tab) {
      tab.text = text;
      tabs = tabs;
      scheduleSave();
    }
  }

  function onTotalChange(tabId: string, total: string) {
    tabTotals[tabId] = total;
    tabTotals = tabTotals;
  }

  function onLocaleChange(locale: UiLocale) {
    uiLocale = locale;
    scheduleSave();
  }

  function copyTotal() {
    if (!activeTotalFormatted) return;
    navigator.clipboard.writeText(activeTotalFormatted).then(() => {
      copiedTotal = true;
      setTimeout(() => { copiedTotal = false; }, 1200);
    });
  }

  async function exportTab() {
    const tab = activeTab;
    if (!tab || !tab.text) return;
    const defaultName = (tab.title || `tab`).replace(/[^a-zA-Z0-9_\-]/g, "_") + ".txt";
    try {
      await invoke("export_file", { defaultName, content: tab.text });
    } catch (e) {
      // User cancelled or error — silently ignore
    }
  }

  async function fetchRates() {
    try {
      const resp = await fetch("https://open.er-api.com/v6/latest/USD");
      const data = await resp.json();
      if (data.result === "success") {
        await invoke("update_rates", { rates: data.rates });
      }
    } catch (e) {
      console.warn("Failed to fetch rates, using fallback:", e);
    }
  }

  onMount(async () => {
    await loadPersistedState();
    fetchRates();

    function onGlobalKeydown(e: KeyboardEvent) {
      // Ctrl+Shift+T — reopen last closed tab
      if (e.ctrlKey && e.shiftKey && e.key === "T") {
        e.preventDefault();
        reopenTab();
        return;
      }

      // Ctrl+W — close active tab
      if (e.ctrlKey && !e.shiftKey && e.key === "w") {
        e.preventDefault();
        closeTab(activeTabId);
        return;
      }

      // Ctrl+T — new tab
      if (e.ctrlKey && !e.shiftKey && e.key === "t") {
        e.preventDefault();
        createTab();
        return;
      }

      // Ctrl+S — export tab
      if (e.ctrlKey && !e.shiftKey && e.key === "s") {
        e.preventDefault();
        exportTab();
        return;
      }

      // Ctrl+Tab — next tab
      if (e.ctrlKey && !e.shiftKey && e.key === "Tab") {
        e.preventDefault();
        const idx = tabs.findIndex(t => t.id === activeTabId);
        const nextIdx = (idx + 1) % tabs.length;
        selectTab(tabs[nextIdx].id);
        return;
      }

      // Ctrl+Shift+Tab — previous tab
      if (e.ctrlKey && e.shiftKey && e.key === "Tab") {
        e.preventDefault();
        const idx = tabs.findIndex(t => t.id === activeTabId);
        const prevIdx = (idx - 1 + tabs.length) % tabs.length;
        selectTab(tabs[prevIdx].id);
        return;
      }

      // Ctrl+1..9 — switch to tab by index
      if (e.ctrlKey && !e.shiftKey && e.key >= "1" && e.key <= "9") {
        e.preventDefault();
        const targetIdx = parseInt(e.key) - 1;
        if (targetIdx < tabs.length) {
          selectTab(tabs[targetIdx].id);
        }
        return;
      }
    }

    document.addEventListener("keydown", onGlobalKeydown);
    return () => document.removeEventListener("keydown", onGlobalKeydown);
  });
</script>

<main>
  <TabBar
    tabs={displayTabs}
    {activeTabId}
    onSelect={selectTab}
    onClose={closeTab}
    onNew={createTab}
    onRename={renameTab}
    onReorder={reorderTabs}
  />

  <div class="editor-container">
    {#if loaded}
      {#each tabs as tab (tab.id)}
        <div class="editor-wrapper" class:active={tab.id === activeTabId}>
          <EditorPane
            tabId={tab.id}
            initialText={tab.text}
            {uiLocale}
            {onTextChange}
            {onTotalChange}
          />
        </div>
      {/each}
    {/if}
  </div>

  <div class="footer">
    <div class="footer-left">
      <button class="settings-btn" onclick={() => showSettings = !showSettings} title="Settings">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
          <path d="M8 10a2 2 0 100-4 2 2 0 000 4z" stroke="currentColor" stroke-width="1.3"/>
          <path d="M13.5 8c0-.3-.2-.6-.4-.8l1-1.6-.8-1.4-1.8.4c-.4-.3-.8-.6-1.3-.7L9.8 2H8.2l-.4 1.9c-.5.1-.9.4-1.3.7l-1.8-.4-.8 1.4 1 1.6c-.2.2-.4.5-.4.8s.2.6.4.8l-1 1.6.8 1.4 1.8-.4c.4.3.8.6 1.3.7L8.2 14h1.6l.4-1.9c.5-.1.9-.4 1.3-.7l1.8.4.8-1.4-1-1.6c.2-.2.4-.5.4-.8z" stroke="currentColor" stroke-width="1.3"/>
        </svg>
      </button>
    </div>
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions a11y_no_noninteractive_element_interactions -->
    <div class="total-area" class:clickable={!!activeTotalFormatted} onclick={copyTotal} title={activeTotalFormatted ? strings.copyTooltip : ""} role="button" tabindex="-1">
      {#if copiedTotal}
        <span class="total-copied">{strings.copied}</span>
      {:else if activeTotalFormatted}
        <span class="total-label">{strings.total}</span>
        <span class="total-value">{activeTotalFormatted}</span>
      {/if}
    </div>
  </div>

  {#if showSettings}
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions a11y_interactive_supports_focus -->
    <div class="settings-overlay" role="dialog" aria-modal="true" tabindex="-1" onclick={() => showSettings = false}>
      <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions a11y_no_noninteractive_element_interactions -->
      <div class="settings-panel" role="none" onclick={(e) => e.stopPropagation()}>
        <div class="settings-section">
          <span class="settings-label">{uiLocale === "pt-br" ? "Idioma" : "Language"}</span>
          <div class="locale-selector">
            <button class="locale-btn" class:active={uiLocale === "pt-br"} onclick={() => onLocaleChange("pt-br")}>🇧🇷 Português</button>
            <button class="locale-btn" class:active={uiLocale === "en"} onclick={() => onLocaleChange("en")}>🇺🇸 English</button>
          </div>
        </div>
        <div class="settings-section">
          <span class="settings-label">{uiLocale === "pt-br" ? "Tema" : "Theme"}</span>
          <div class="locale-selector">
            <button class="locale-btn" class:active={theme === "dark"} onclick={() => { theme = "dark"; applyTheme(theme); scheduleSave(); }}>🌙 Dark</button>
            <button class="locale-btn" class:active={theme === "light"} onclick={() => { theme = "light"; applyTheme(theme); scheduleSave(); }}>☀ Light</button>
          </div>
        </div>
        <div class="settings-section">
          <button class="settings-action" onclick={() => { showSettings = false; showHelp = true; }}>
            {strings.help} — ?
          </button>
        </div>
      </div>
    </div>
  {/if}

  {#if showHelp}
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions a11y_interactive_supports_focus -->
    <div class="help-overlay" role="dialog" aria-modal="true" tabindex="-1" onclick={() => showHelp = false}>
      <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions a11y_no_noninteractive_element_interactions -->
      <div class="help-modal" role="none" onclick={(e) => e.stopPropagation()}>
        <div class="help-header">
          <h3>{strings.helpTitle}</h3>
          <button class="help-close" onclick={() => showHelp = false}>✕</button>
        </div>
        <div class="help-content">
          {#each strings.helpSections as section}
            <div class="help-section">
              <h4>{section.title}</h4>
              {#each section.examples as example}
                <code>{example}</code>
              {/each}
            </div>
          {/each}
        </div>
      </div>
    </div>
  {/if}
</main>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: "JetBrains Mono", "Fira Code", "SF Mono", "Cascadia Code", "Courier New", monospace;
    background: var(--bg-base);
    color: var(--text-primary);
    overflow: hidden;
  }

  main {
    height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .editor-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    position: relative;
  }

  .editor-wrapper {
    display: none;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
  }

  .editor-wrapper.active {
    display: flex;
  }

  .footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 16px;
    border-top: 1px solid var(--border);
    background: var(--bg-deep);
    font-size: 14px;
  }

  .footer-left {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .total-area {
    display: flex;
    gap: 8px;
    align-items: center;
    padding: 2px 8px;
    border-radius: 4px;
    transition: background 0.15s;
  }

  .total-area.clickable {
    cursor: pointer;
  }

  .total-area.clickable:hover {
    background: var(--bg-elevated);
  }

  .total-label { color: var(--text-muted); }
  .total-value { color: var(--yellow); font-weight: 600; }
  .total-copied { color: var(--green); font-size: 12px; }

  .settings-btn {
    width: 28px;
    height: 28px;
    border-radius: 6px;
    border: 1px solid var(--border-strong);
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
  }

  .settings-btn:hover {
    border-color: var(--accent);
    color: var(--accent);
    background: var(--bg-elevated);
  }

  /* Settings panel */
  .settings-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.3);
    display: flex;
    align-items: flex-end;
    justify-content: flex-start;
    z-index: 90;
    padding: 0 0 50px 12px;
  }

  .settings-panel {
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 12px 16px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
    min-width: 200px;
  }

  .settings-section {
    margin-bottom: 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .settings-section:last-child {
    margin-bottom: 0;
  }

  .settings-label {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
    font-weight: 600;
  }

  .locale-selector {
    display: flex;
    gap: 4px;
  }

  .locale-btn {
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 6px;
    font-size: 12px;
    cursor: pointer;
    padding: 4px 10px;
    color: var(--text-secondary);
    transition: all 0.15s;
    font-family: inherit;
  }

  .locale-btn:hover {
    background: var(--bg-elevated);
    color: var(--text-primary);
  }

  .locale-btn.active {
    border-color: var(--accent);
    background: var(--bg-elevated);
    color: var(--text-primary);
  }

  .settings-action {
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 6px;
    font-size: 12px;
    cursor: pointer;
    padding: 6px 10px;
    color: var(--text-secondary);
    font-family: inherit;
    text-align: left;
    transition: all 0.15s;
  }

  .settings-action:hover {
    background: var(--bg-elevated);
    color: var(--text-primary);
  }

  /* Help overlay & modal */
  .help-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .help-modal {
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: 12px;
    width: 420px;
    max-height: 80vh;
    overflow-y: auto;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  }

  .help-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border);
  }

  .help-header h3 {
    margin: 0;
    font-size: 16px;
    color: var(--text-primary);
    font-weight: 600;
  }

  .help-close {
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 18px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
  }

  .help-close:hover {
    color: var(--red);
    background: var(--bg-elevated);
  }

  .help-content {
    padding: 16px 20px;
  }

  .help-section {
    margin-bottom: 16px;
  }

  .help-section h4 {
    margin: 0 0 6px 0;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
    font-weight: 600;
  }

  .help-section code {
    display: block;
    padding: 4px 10px;
    margin: 3px 0;
    background: var(--bg-deep);
    border-radius: 4px;
    font-size: 13px;
    color: var(--green);
    font-family: inherit;
  }
</style>

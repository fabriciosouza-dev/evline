<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { t, type UiLocale } from "./i18n";

  interface LineResult {
    line: number;
    input: string;
    output: string;
    result_type: any;
    numeric_value: number | null;
  }

  interface EvalResponse {
    results: LineResult[];
    total: number | null;
    total_formatted: string;
    locale: string;
  }

  let {
    tabId,
    initialText = "",
    uiLocale = "pt-br" as UiLocale,
    onTextChange,
    onTotalChange,
  }: {
    tabId: string;
    initialText?: string;
    uiLocale?: UiLocale;
    onTextChange?: (tabId: string, text: string) => void;
    onTotalChange?: (tabId: string, total: string) => void;
  } = $props();

  let strings = $derived(t(uiLocale));

  let text = $state("");
  let results: LineResult[] = $state([]);
  let totalFormatted = $state("");
  let textareaEl: HTMLTextAreaElement;
  let highlightEl: HTMLDivElement;
  let debounceTimer: any;
  let lineCount = $state(1);
  let copiedIndex = $state(-1);
  let initialized = false;

  // Autocomplete state
  let showAutocomplete = $state(false);
  let autocompleteItems: string[] = $state([]);
  let autocompleteIndex = $state(0);
  let currentWord = "";
  let autocompleteTop = $state(0);
  let autocompleteLeft = $state(0);

  // Undo/Redo stack
  let undoStack: { text: string; cursor: number }[] = [];
  let redoStack: { text: string; cursor: number }[] = [];
  let undoTimer: any;
  let lastSavedText = "";

  function pushUndo() {
    if (text !== lastSavedText) {
      undoStack.push({ text: lastSavedText, cursor: textareaEl?.selectionStart || 0 });
      if (undoStack.length > 100) undoStack.shift();
      redoStack = [];
      lastSavedText = text;
    }
  }

  function scheduleUndoSnapshot() {
    clearTimeout(undoTimer);
    undoTimer = setTimeout(pushUndo, 300);
  }

  function undo() {
    pushUndo(); // save current state first
    const entry = undoStack.pop();
    if (entry) {
      redoStack.push({ text, cursor: textareaEl.selectionStart });
      text = entry.text;
      textareaEl.value = text;
      textareaEl.setSelectionRange(entry.cursor, entry.cursor);
      lineCount = text.split("\n").length;
      updateHighlight();
      clearTimeout(debounceTimer);
      debounceTimer = setTimeout(evaluateText, 50);
      onTextChange?.(tabId, text);
      lastSavedText = text;
    }
  }

  function redo() {
    const entry = redoStack.pop();
    if (entry) {
      undoStack.push({ text, cursor: textareaEl.selectionStart });
      text = entry.text;
      textareaEl.value = text;
      textareaEl.setSelectionRange(entry.cursor, entry.cursor);
      lineCount = text.split("\n").length;
      updateHighlight();
      clearTimeout(debounceTimer);
      debounceTimer = setTimeout(evaluateText, 50);
      onTextChange?.(tabId, text);
      lastSavedText = text;
    }
  }

  const SUGGESTIONS = [
    "USD", "EUR", "BRL", "GBP", "JPY", "CNY", "CAD", "AUD", "CHF",
    "ARS", "CLP", "COP", "MXN",
    "dollars", "euros", "pounds", "yen",
    "dólares", "reais", "libras", "ienes",
    "km", "miles", "milhas", "meters", "metros", "feet", "inches",
    "cm", "mm", "yards",
    "kg", "quilos", "pounds", "libras", "grams", "gramas",
    "liters", "litros", "ml", "cups", "xícaras",
    "teaspoons", "tablespoons",
    "celsius", "fahrenheit", "kelvin",
    "hours", "horas", "minutes", "minutos", "seconds", "segundos",
    "days", "dias", "weeks", "semanas", "months", "meses", "years", "anos",
    "GB", "MB", "KB", "TB",
    "in", "em", "para",
    "today", "hoje",
    "sum", "soma", "total",
    "prev", "anterior",
    "average", "avg", "média",
    "discount", "desconto",
    "times", "vezes", "plus", "mais", "minus", "menos", "divided", "dividido",
    "sqrt(", "log(", "round(", "abs(", "floor(", "ceil(",
    "fromunix(", "tounix(", "deunix(", "paraunix(",
    "now", "agora", "epoch", "timestamp",
    "of what is", "de quanto é",
  ];

  const CURRENCY_CODES = ["USD", "EUR", "GBP", "BRL", "JPY", "CNY", "CAD", "AUD", "CHF", "ARS", "CLP", "COP", "MXN"];
  const CURRENCY_NAMES_HL = ["dollar", "dollars", "dólar", "dólares", "dolar", "dolares", "euro", "euros", "pound", "pounds", "libra", "libras", "real", "reais", "yen", "iene"];
  const KEYWORDS = ["in", "to", "em", "para", "today", "hoje", "sum", "total", "soma", "of what is", "de quanto é", "prev", "anterior", "average", "avg", "média", "discount", "desconto", "of", "on", "off", "sobre"];
  const OPERATORS_WORDS = ["times", "vezes", "plus", "mais", "minus", "menos", "divided", "dividido", "mod"];
  const UNITS = [
    "km", "miles", "meters", "feet", "inches", "yards", "cm", "mm",
    "milhas", "metros", "quilômetros", "centímetros",
    "kg", "pounds", "grams", "lbs", "oz", "quilos", "libras", "gramas",
    "liters", "ml", "cups", "tea spoons", "tablespoons", "litros", "xícaras",
    "hours", "minutes", "seconds", "days", "weeks", "months", "years",
    "horas", "minutos", "segundos", "dias", "semanas", "meses", "anos",
    "celsius", "fahrenheit", "kelvin",
    "GB", "MB", "KB", "TB",
  ];
  const FUNCTIONS = ["sqrt", "log", "log2", "sin", "cos", "tan", "abs", "round", "floor", "ceil", "exp"];

  function onInput() {
    text = textareaEl.value;
    lineCount = text.split("\n").length;
    updateHighlight();
    updateAutocomplete();
    scheduleUndoSnapshot();
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(evaluateText, 50);
    onTextChange?.(tabId, text);
  }

  function onKeydown(e: KeyboardEvent) {
    // Undo/Redo
    if (e.ctrlKey && !e.shiftKey && e.key === "z") {
      e.preventDefault();
      undo();
      return;
    }
    if ((e.ctrlKey && e.key === "y") || (e.ctrlKey && e.shiftKey && e.key === "Z")) {
      e.preventDefault();
      redo();
      return;
    }

    if (showAutocomplete) {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        autocompleteIndex = (autocompleteIndex + 1) % autocompleteItems.length;
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        autocompleteIndex = (autocompleteIndex - 1 + autocompleteItems.length) % autocompleteItems.length;
      } else if (e.key === "Tab" || e.key === "Enter") {
        if (autocompleteItems.length > 0) {
          e.preventDefault();
          applyAutocomplete(autocompleteItems[autocompleteIndex]);
        }
      } else if (e.key === "Escape") {
        showAutocomplete = false;
      }
    }
  }

  function updateAutocomplete() {
    const cursorPos = textareaEl.selectionStart;
    const textBeforeCursor = text.substring(0, cursorPos);
    const currentLine = textBeforeCursor.split("\n").pop() || "";
    const lineIndex = textBeforeCursor.split("\n").length - 1;

    const match = currentLine.match(/[a-zA-ZÀ-ÿ]+$/);
    currentWord = match ? match[0] : "";

    if (currentWord.length < 2) {
      showAutocomplete = false;
      return;
    }

    const lower = currentWord.toLowerCase();

    autocompleteItems = SUGGESTIONS.filter(s => {
      const sl = s.toLowerCase();
      return sl.startsWith(lower) && sl !== lower;
    }).slice(0, 6);

    if (autocompleteItems.length > 0) {
      showAutocomplete = true;
      autocompleteIndex = 0;
      autocompleteTop = (lineIndex + 1) * 28 + 16 + 4;
      autocompleteLeft = currentLine.length * 9;
      if (autocompleteLeft < 20) autocompleteLeft = 20;
      if (autocompleteLeft > 300) autocompleteLeft = 300;
    } else {
      showAutocomplete = false;
    }
  }

  function applyAutocomplete(item: string) {
    const cursorPos = textareaEl.selectionStart;
    const before = text.substring(0, cursorPos);
    const after = text.substring(cursorPos);

    const newBefore = before.substring(0, before.length - currentWord.length) + item;
    text = newBefore + after;
    textareaEl.value = text;

    const newPos = newBefore.length;
    textareaEl.setSelectionRange(newPos, newPos);

    showAutocomplete = false;
    lineCount = text.split("\n").length;
    updateHighlight();
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(evaluateText, 50);
    onTextChange?.(tabId, text);
  }

  function updateHighlight() {
    if (!highlightEl) return;
    const lines = text.split("\n");
    highlightEl.innerHTML = lines.map(line => highlightLine(line)).join("\n");
  }

  function highlightLine(line: string): string {
    if (!line.trim()) return "&nbsp;";
    if (line.trim().startsWith("//") || line.trim().startsWith("#")) {
      return `<span class="hl-comment">${escapeHtml(line)}</span>`;
    }

    let result = escapeHtml(line);

    result = result.replace(/(\$|R\$|€|£|¥)/g, '<span class="hl-currency-sym">$1</span>');

    for (const code of CURRENCY_CODES) {
      const regex = new RegExp(`\\b(${code})\\b`, "g");
      result = result.replace(regex, '<span class="hl-currency">$1</span>');
    }

    for (const name of CURRENCY_NAMES_HL) {
      const regex = new RegExp(`\\b(${escapeRegex(name)})\\b`, "gi");
      result = result.replace(regex, '<span class="hl-currency">$1</span>');
    }

    for (const kw of KEYWORDS) {
      const regex = new RegExp(`\\b(${escapeRegex(kw)})\\b`, "gi");
      result = result.replace(regex, '<span class="hl-keyword">$1</span>');
    }

    for (const op of OPERATORS_WORDS) {
      const regex = new RegExp(`\\b(${escapeRegex(op)})\\b`, "gi");
      result = result.replace(regex, '<span class="hl-operator">$1</span>');
    }

    for (const fn of FUNCTIONS) {
      const regex = new RegExp(`\\b(${fn})(\\()`, "g");
      result = result.replace(regex, '<span class="hl-function">$1</span>$2');
    }

    for (const unit of UNITS) {
      if (unit.length < 2) continue;
      const regex = new RegExp(`\\b(${escapeRegex(unit)})\\b`, "gi");
      result = result.replace(regex, (match, p1) => {
        return `<span class="hl-unit">${p1}</span>`;
      });
    }

    result = result.replace(/(\d+%)/g, '<span class="hl-percent">$1</span>');

    return result;
  }

  function escapeHtml(s: string): string {
    return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  }

  function escapeRegex(s: string): string {
    return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  }

  async function evaluateText() {
    try {
      const response = await invoke<EvalResponse>("evaluate", { tabId, text, uiLocale });
      results = response.results;
      totalFormatted = response.total_formatted;
      onTotalChange?.(tabId, totalFormatted);
    } catch (err) {
      console.error("Evaluation error:", err);
    }
  }

  function copyResult(output: string, index: number) {
    if (!output) return;
    navigator.clipboard.writeText(output).then(() => {
      copiedIndex = index;
      setTimeout(() => { copiedIndex = -1; }, 1200);
    });
  }

  function getResultClass(rt: any): string {
    if (typeof rt === "string") {
      switch (rt) {
        case "Number": return "number";
        case "Date": return "date";
        case "Error": return "error";
        case "Comment": return "comment";
        case "Empty": return "empty";
        default: return "";
      }
    }
    if (rt && "Currency" in rt) return "currency";
    if (rt && "Unit" in rt) return "unit";
    return "";
  }

  function onScroll() {
    // Textarea overflow is hidden — scroll is on parent .editor
    // Only sync highlight layer with textarea (if textarea somehow scrolls)
    if (highlightEl && textareaEl) {
      highlightEl.scrollTop = textareaEl.scrollTop;
      highlightEl.scrollLeft = textareaEl.scrollLeft;
    }
  }

  export function focus() {
    textareaEl?.focus();
  }

  export function getText(): string {
    return text;
  }

  export function getTotalFormatted(): string {
    return totalFormatted;
  }

  onMount(async () => {
    text = initialText;
    lastSavedText = text;
    // Wait for DOM to be ready
    await new Promise(r => setTimeout(r, 0));
    if (textareaEl) {
      textareaEl.value = text;
    }
    if (text) {
      lineCount = text.split("\n").length;
      updateHighlight();
      evaluateText();
    }
    initialized = true;
    textareaEl?.focus();
  });
</script>

<div class="editor">
  <div class="editor-scroll">
    <div class="gutter">
      {#each Array(Math.max(lineCount, 1)) as _, i}
        <div class="line-number">{i + 1}</div>
      {/each}
    </div>
    <div class="input-pane">
      <div class="highlight-layer" bind:this={highlightEl}></div>
      <textarea
        bind:this={textareaEl}
        oninput={onInput}
        onkeydown={onKeydown}
        onscroll={onScroll}
        placeholder=""
        spellcheck="false"
        autocomplete="off"
      ></textarea>
      {#if showAutocomplete}
        <div class="autocomplete" style="top: {autocompleteTop}px; left: {autocompleteLeft}px;">
          {#each autocompleteItems as item, i}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="autocomplete-item"
              class:active={i === autocompleteIndex}
              onmousedown={(e) => { e.preventDefault(); applyAutocomplete(item); }}
            >
              {item}
            </div>
          {/each}
        </div>
      {/if}
    </div>
    <div class="results-pane">
      {#each results as result, i}
        <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions a11y_no_noninteractive_element_interactions -->
        <div
          class="result-line {getResultClass(result.result_type)}"
          class:copied={copiedIndex === i}
          class:clickable={!!result.output && result.result_type !== "Comment" && result.result_type !== "Empty"}
          style="height: 28px"
          onclick={() => copyResult(result.output, i)}
          title={result.output ? strings.copyTooltip : ""}
          role="button"
          tabindex="-1"
        >
          {copiedIndex === i ? strings.copied : result.output}
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  .editor {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    background: var(--bg-base);
    min-height: 0;
  }

  .editor::-webkit-scrollbar {
    width: 6px;
  }

  .editor::-webkit-scrollbar-track {
    background: transparent;
  }

  .editor::-webkit-scrollbar-thumb {
    background: var(--border-strong);
    border-radius: 3px;
  }

  .editor-scroll {
    display: flex;
    min-height: 100%;
  }

  .gutter {
    padding: 16px 0;
    min-width: 40px;
    text-align: right;
    padding-right: 12px;
    border-right: 1px solid var(--border);
    background: var(--bg-deep);
    user-select: none;
    position: sticky;
    left: 0;
  }

  .line-number {
    height: 28px;
    line-height: 28px;
    font-size: 13px;
    color: var(--text-faint);
  }

  .input-pane {
    flex: 1;
    position: relative;
    min-width: 0;
  }

  .highlight-layer {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    padding: 16px 20px;
    font-family: inherit;
    font-size: 15px;
    line-height: 28px;
    white-space: pre-wrap;
    word-wrap: break-word;
    overflow: hidden;
    pointer-events: none;
    color: transparent;
    z-index: 0;
  }

  .highlight-layer :global(.hl-comment) { color: var(--text-muted); font-style: italic; }
  .highlight-layer :global(.hl-currency) { color: var(--yellow); font-weight: 600; }
  .highlight-layer :global(.hl-currency-sym) { color: var(--yellow); }
  .highlight-layer :global(.hl-keyword) { color: var(--accent); font-weight: 600; }
  .highlight-layer :global(.hl-unit) { color: var(--cyan); }
  .highlight-layer :global(.hl-function) { color: var(--blue); }
  .highlight-layer :global(.hl-operator) { color: var(--orange); font-weight: 600; }
  .highlight-layer :global(.hl-percent) { color: var(--green); }

  textarea {
    position: relative;
    width: 100%;
    min-height: 100%;
    background: transparent;
    border: none;
    outline: none;
    color: var(--text-primary);
    caret-color: var(--caret);
    font-family: inherit;
    font-size: 15px;
    line-height: 28px;
    padding: 16px 20px;
    resize: none;
    overflow: hidden;
    white-space: pre-wrap;
    word-wrap: break-word;
    tab-size: 2;
    z-index: 1;
    box-sizing: border-box;
  }

  textarea::placeholder { color: var(--text-muted); }

  .autocomplete {
    position: absolute;
    background: var(--bg-elevated);
    border: 1px solid var(--border-strong);
    border-radius: 6px;
    padding: 4px 0;
    z-index: 10;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    max-width: 280px;
  }

  .autocomplete-item {
    padding: 6px 14px;
    font-size: 13px;
    color: var(--text-primary);
    cursor: pointer;
    white-space: nowrap;
  }

  .autocomplete-item:hover,
  .autocomplete-item.active {
    background: var(--border-strong);
    color: var(--caret);
  }

  .results-pane {
    width: 200px;
    min-width: 150px;
    padding: 16px 20px;
    border-left: 1px solid var(--border);
    display: flex;
    flex-direction: column;
  }

  .result-line {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    font-size: 15px;
    line-height: 28px;
    text-align: right;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    border-radius: 4px;
    padding: 0 6px;
    transition: background 0.15s;
  }

  .result-line.clickable { cursor: pointer; }
  .result-line.clickable:hover { background: var(--bg-elevated); }
  .result-line.copied { color: var(--green) !important; font-size: 12px; }
  .result-line.number { color: var(--green); }
  .result-line.currency { color: var(--yellow); }
  .result-line.unit { color: var(--cyan); }
  .result-line.date { color: var(--accent); }
  .result-line.error { color: var(--red); font-size: 12px; }
  .result-line.comment, .result-line.empty { color: transparent; }
</style>

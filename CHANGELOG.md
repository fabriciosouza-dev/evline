# Changelog

All notable changes to Evline will be documented in this file.

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1] - 2026-08-22

### Added
- Local timezone for `now`/`agora` instead of UTC (PR #1 by @danilocgomesdev)
- Compound duration arithmetic: `today + 2 months 3 days`

## [0.3.0] - 2026-08-22

### Added
- HTTP status code lookup: `http 200`, `status 404`
- Port lookup (bidirectional): `port ssh` → 22, `porta 3306` → MySQL
- Resizable results pane (drag border)
- Tooltip shows full text for truncated results
- Easter eggs with visual effects (matrix, thanos, barrel roll, gravity, tilt, blink)
- Toast notification system for easter eggs

## [0.2.0] - 2026-08-21

### Added
- Date format conversion: `08/21/2026 to BR`, `21/08/2026 to ISO`
- `now`/`agora` returns full datetime, `epoch`/`timestamp` returns raw unix
- `tounix()` accepts time component
- `today +/- hours/minutes` returns date+time
- Undo/Redo (Ctrl+Z / Ctrl+Y) with custom history stack
- Settings panel (gear icon) replaces footer clutter
- Drag & drop tabs to reorder
- Ctrl+S export tab as .txt
- Click total to copy to clipboard
- Dark/Light theme toggle (Catppuccin Mocha/Latte)
- CSS variables for theming
- Unified scroll (gutter, editor, results scroll together)
- Quick Guide with shortcuts section

### Fixed
- Parser accepts currency symbols ($, €, £, ¥, R$) mid-expression
- `prev in BRL` works (missing space in concatenation)
- Trailing words after % treated as labels (e.g., `15% emergency`)
- Date conversion no longer intercepts non-date expressions
- CSS variables with defaults in index.html prevent flash on load

## [0.1.0] - 2026-08-21

### Added
- Notepad-style calculator — evaluate every line
- Bilingual engine (English + Portuguese simultaneously)
- Live currency conversion with auto-updated rates
- Unit conversion (distance, weight, volume, temperature, time, data)
- Variables and labels
- Totals: sum, average, prev/anterior
- Date arithmetic: today +/- days/weeks/months/years
- Epoch/timestamp conversion (fromunix, tounix)
- Multiple tabs with independent contexts
- Tab persistence in ~/.evline/state.json
- Syntax highlighting and autocomplete
- Click results to copy
- Hex/Binary/Octal support
- Locale selector (PT-BR / EN) for UI and error messages
- Comments with // or #

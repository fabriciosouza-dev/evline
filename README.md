# Evline

> A notepad-style calculator for Linux — evaluate every line.

**[Leia em Português](./README.pt-br.md)**

![Evline Screenshot](https://img.shields.io/badge/platform-Linux-blue) ![License](https://img.shields.io/badge/license-MIT-green)

Evline is a desktop calculator that works like a text editor. Type expressions naturally — one per line — and see results instantly on the right side. No buttons, no equals sign. Just type and think.

## What makes it powerful

- **Natural language math** — write `200 + 10%`, `$50 - 5% discount`, `8 times 9`
- **Bilingual engine** — understands both English and Portuguese simultaneously (`hoje + 17 dias`, `today + 3 months`)
- **Live currency conversion** — `$100 in EUR`, `50 pounds em reais` (rates updated on launch)
- **Unit conversion** — `10 km in miles`, `100 celsius in fahrenheit`, `20 ml in tea spoons`
- **Variables** — `price = 100` then use `price + 15%` on the next line
- **Totals and aggregation** — `sum`, `average`, reference previous line with `prev`
- **Date arithmetic** — `today + 3 months`, `hoje + 17 dias`
- **Multiple tabs** — independent calculation contexts, drag to reorder, rename with double-click
- **Persistent state** — tabs, content, and settings survive between sessions (`~/.evline/`)
- **Dark/Light theme** — Catppuccin Mocha and Latte
- **Syntax highlighting and autocomplete** — as you type
- **Click to copy** — click any result or the total to copy to clipboard
- **Export** — Ctrl+S saves the current tab as a `.txt` file

## Keyboard shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl+T | New tab |
| Ctrl+W | Close tab |
| Ctrl+Shift+T | Reopen closed tab |
| Ctrl+Tab | Next tab |
| Ctrl+Shift+Tab | Previous tab |
| Ctrl+1..9 | Jump to tab N |
| Ctrl+S | Export tab as .txt |

## Installation

### From .deb package (Debian/Ubuntu)

```bash
# Download the latest release from GitHub Releases, then:
sudo dpkg -i Evline_0.1.0_amd64.deb
```

### Build from source

**Prerequisites:**
- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (18+)
- System dependencies for Tauri on Linux:

```bash
# Debian/Ubuntu
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

**Build:**

```bash
git clone https://github.com/fabriciosouza-dev/evline.git
cd evline
npm install
npm run tauri build
```

The `.deb` will be at `src-tauri/target/release/bundle/deb/`.

**Run in development:**

```bash
npm run tauri dev
```

## How it works

Each line is independently evaluated. The engine parses natural language expressions, handles operator precedence, resolves variables, converts currencies and units, and formats results according to context.

```
rent = 1200
groceries = 450
transport = 180
rent + groceries + transport        → 1,830
sum                                  → 1,830
prev in EUR                          → 1,647.00 (live rate)
```

## Tech stack

- **Frontend:** Svelte 5, Vite
- **Backend:** Rust (Tauri 2)
- **Themes:** Catppuccin Mocha / Latte
- **Persistence:** JSON at `~/.evline/state.json`

## License

MIT

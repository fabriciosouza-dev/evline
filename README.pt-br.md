# Evline

> Uma calculadora estilo bloco de notas para Linux — avalie cada linha.

**[Read in English](./README.md)**

![Evline Screenshot](https://img.shields.io/badge/plataforma-Linux-blue) ![License](https://img.shields.io/badge/licença-MIT-green)

Evline é uma calculadora desktop que funciona como um editor de texto. Digite expressões naturalmente — uma por linha — e veja os resultados instantaneamente do lado direito. Sem botões, sem sinal de igual. Apenas digite e pense.

## O que torna poderoso

- **Matemática em linguagem natural** — escreva `200 + 10%`, `R$50 - 5% desconto`, `8 vezes 9`
- **Motor bilíngue** — entende português e inglês simultaneamente (`hoje + 17 dias`, `today + 3 months`)
- **Conversão de moedas em tempo real** — `$100 in EUR`, `50 pounds em reais` (taxas atualizadas ao abrir)
- **Conversão de unidades** — `10 km em milhas`, `100 celsius in fahrenheit`, `20 ml in tea spoons`
- **Variáveis** — `preco = 100` e depois use `preco + 15%` na próxima linha
- **Totais e agregação** — `soma`, `média`, referência à linha anterior com `anterior`
- **Aritmética de datas** — `hoje + 3 meses`, `today + 17 days`
- **Múltiplas abas** — contextos de cálculo independentes, arraste para reordenar, renomeie com duplo-clique
- **Estado persistente** — abas, conteúdo e configurações sobrevivem entre sessões (`~/.evline/`)
- **Tema escuro/claro** — Catppuccin Mocha e Latte
- **Syntax highlighting e autocomplete** — enquanto você digita
- **Clique para copiar** — clique em qualquer resultado ou no total para copiar
- **Exportar** — Ctrl+S salva a aba atual como arquivo `.txt`

## Atalhos de teclado

| Atalho | Ação |
|--------|------|
| Ctrl+T | Nova aba |
| Ctrl+W | Fechar aba |
| Ctrl+Shift+T | Reabrir aba fechada |
| Ctrl+Tab | Próxima aba |
| Ctrl+Shift+Tab | Aba anterior |
| Ctrl+1..9 | Ir para aba N |
| Ctrl+S | Exportar aba como .txt |

## Instalação

### Via pacote .deb (Debian/Ubuntu)

```bash
# Baixe o release mais recente do GitHub Releases, depois:
sudo dpkg -i Evline_0.1.0_amd64.deb
```

### Compilar do código-fonte

**Pré-requisitos:**
- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (18+)
- Dependências de sistema para Tauri no Linux:

```bash
# Debian/Ubuntu
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

**Compilar:**

```bash
git clone https://github.com/fabriciosouza-dev/evline.git
cd evline
npm install
npm run tauri build
```

O `.deb` estará em `src-tauri/target/release/bundle/deb/`.

**Rodar em modo desenvolvimento:**

```bash
npm run tauri dev
```

## Como funciona

Cada linha é avaliada independentemente. O motor parseia expressões em linguagem natural, resolve precedência de operadores, variáveis, converte moedas e unidades, e formata resultados de acordo com o contexto.

```
aluguel = 1200
mercado = 450
transporte = 180
aluguel + mercado + transporte      → 1.830
soma                                 → 1.830
anterior em USD                      → 328.00 (taxa ao vivo)
```

## Stack tecnológica

- **Frontend:** Svelte 5, Vite
- **Backend:** Rust (Tauri 2)
- **Temas:** Catppuccin Mocha / Latte
- **Persistência:** JSON em `~/.evline/state.json`

## Licença

MIT

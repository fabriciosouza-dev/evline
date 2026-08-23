# Evline

### Pare de gastar créditos com IA para contas simples. Apenas digite.

**[Read in English](./README.md)**

---

Evline é uma calculadora estilo bloco de notas para Linux. Digite expressões naturalmente — uma por linha — e veja os resultados instantaneamente. Sem botões, sem sinal de igual, sem trocar de janela. A resposta está sempre ali.

![Evline Editor](./assets/screenshot-editor.png)

## Por que Evline?

Você não precisa do ChatGPT para dividir uma conta. Não precisa de uma planilha para organizar o mês. Não precisa abrir o navegador para converter moedas.

Evline substitui a ginástica mental do "deixa eu calcular rapidinho..." por um bloco de notas que **pensa enquanto você digita**.

```
aluguel = 1200
mercado = 450
transporte = 180
soma                                 → 1.830
anterior em USD                      → 328.00

orçamento viagem:
passagens = $1200
hotel = 5 * $89
alimentação = 7 * $45
soma                                 → 2,060
anterior em BRL                      → R$ 11.330,00

// Referência rápida
http 404                             → 404 Not Found [client error]
status 201                           → 201 Created [ok]
porta ssh                            → 22 (TCP)
porta redis                          → 6379 (TCP)
```

## Funcionalidades

![Evline Guia Rápido](./assets/screenshot-guide.png)

- **Matemática em linguagem natural** — `200 + 10%`, `R$50 - 5% desconto`, `8 vezes 9`
- **Motor bilíngue** — entende português e inglês simultaneamente
- **Conversão de moedas em tempo real** — `$100 in EUR`, `50 pounds em reais`
- **Conversão de unidades** — `10 km em milhas`, `100 celsius in fahrenheit`
- **Variáveis** — `preco = 100` e depois `preco + 15%`
- **Totais** — `soma`, `média`, `anterior`
- **Aritmética de datas** — `hoje + 3 meses`, `today + 17 days`, `agora` mostra data e hora
- **Conversão de formato de data** — `08/21/2026 to BR`, `21/08/2026 to ISO`
- **Códigos HTTP** — `http 200`, `status 404` → referência instantânea
- **Lookup de portas** — `port ssh` → 22, `porta 3306` → MySQL (bidirecional)
- **Múltiplas abas** — arraste para reordenar, Ctrl+T para criar, duplo-clique para renomear
- **Painel redimensionável** — arraste a borda para expandir os resultados
- **Estado persistente** — tudo sobrevive entre sessões
- **Tema escuro/claro** — Catppuccin Mocha e Latte
- **Syntax highlighting + autocomplete**
- **Desfazer/Refazer** — Ctrl+Z / Ctrl+Y com histórico completo
- **Clique em qualquer resultado para copiar**
- **Exportar** — Ctrl+S salva como `.txt`

## Atalhos de teclado

| Atalho | Ação |
|--------|------|
| Ctrl+T | Nova aba |
| Ctrl+W | Fechar aba |
| Ctrl+Shift+T | Reabrir aba fechada |
| Ctrl+Tab / Ctrl+Shift+Tab | Navegar abas |
| Ctrl+1..9 | Ir para aba N |
| Ctrl+S | Exportar como .txt |
| Ctrl+Z | Desfazer |
| Ctrl+Y | Refazer |

## Instalar

### Download (Debian/Ubuntu)

Baixe o `.deb` em [Releases](https://github.com/fabriciosouza-dev/evline/releases):

```bash
sudo dpkg -i Evline_0.1.0_amd64.deb
```

### Compilar do código-fonte

```bash
# Pré-requisitos (Debian/Ubuntu)
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libssl-dev libayatana-appindicator3-dev librsvg2-dev

# Compilar
git clone https://github.com/fabriciosouza-dev/evline.git
cd evline
npm install
npm run tauri build

# .deb gerado em src-tauri/target/release/bundle/deb/
```

**Desenvolvimento:**

```bash
npm run tauri dev
```

## Stack tecnológica

| Camada | Tecnologia |
|--------|------------|
| Frontend | Svelte 5, Vite |
| Backend | Rust, Tauri 2 |
| Temas | Catppuccin Mocha / Latte |
| Persistência | `~/.evline/state.json` |

## Contribuir

PRs são bem-vindos. Rode `npm run tauri dev` para começar.

## Licença

MIT

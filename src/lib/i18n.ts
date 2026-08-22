export type UiLocale = "pt-br" | "en";

interface HelpSection {
  title: string;
  examples: string[];
}

interface UiStrings {
  helpTitle: string;
  helpSections: HelpSection[];
  copyTooltip: string;
  copied: string;
  total: string;
  newTab: string;
  closeTab: string;
  help: string;
  tabDefault: string;
}

const ptBr: UiStrings = {
  helpTitle: "Evline — Guia Rápido",
  helpSections: [
    {
      title: "Aritmética",
      examples: ["2 + 3 * (4 - 1)", "sqrt(144)", "2 ^ 10"],
    },
    {
      title: "Variáveis",
      examples: ["preco = 100", "preco + 15%"],
    },
    {
      title: "Prev / Anterior",
      examples: ["Cost: $20 + 56", "prev - 5% discount"],
    },
    {
      title: "Labels",
      examples: ["Preço: R$7 * 4", "Taxa: 10 + 5"],
    },
    {
      title: "Moedas",
      examples: ["$100 in EUR", "50 pounds em reais", "20 dólares in euros", "R$200 + 10%"],
    },
    {
      title: "Operadores",
      examples: ["8 times 9 / 8 vezes 9", "10 plus 5 / 10 mais 5", "20 minus 3 / 20 menos 3"],
    },
    {
      title: "Unidades",
      examples: ["10 km em milhas", "20 ml in tea spoons", "100 celsius in fahrenheit"],
    },
    {
      title: "Datas",
      examples: ["hoje + 17 dias", "today + 3 months"],
    },
    {
      title: "Porcentagem",
      examples: ["200 + 10%", "20% of $10", "5% on $30 (adicionar)", "10% off 100 (subtrair)", "$50 - 5% discount"],
    },
    {
      title: "Totais",
      examples: ["soma / sum / total", "média / average / avg", "total em USD"],
    },
    {
      title: "Hex / Bin / Oct",
      examples: ["0xFF → 255", "0b1010 → 10", "0o77 → 63"],
    },
    {
      title: "Comentários",
      examples: ["// isso é um comentário", "# isso também"],
    },
    {
      title: "Epoch / Timestamp",
      examples: ["now / agora → timestamp atual", "fromunix(1446587186)", "tounix(21/08/2026)", "deunix(1446587186000)"],
    },
  ],
  copyTooltip: "Clique para copiar",
  copied: "✓ copiado",
  total: "Total",
  newTab: "Nova aba",
  closeTab: "Fechar aba",
  help: "Ajuda",
  tabDefault: "Aba",
};

const en: UiStrings = {
  helpTitle: "Evline — Quick Guide",
  helpSections: [
    {
      title: "Arithmetic",
      examples: ["2 + 3 * (4 - 1)", "sqrt(144)", "2 ^ 10"],
    },
    {
      title: "Variables",
      examples: ["price = 100", "price + 15%"],
    },
    {
      title: "Prev / Previous",
      examples: ["Cost: $20 + 56", "prev - 5% discount"],
    },
    {
      title: "Labels",
      examples: ["Price: $7 * 4", "Tax: 10 + 5"],
    },
    {
      title: "Currencies",
      examples: ["$100 in EUR", "50 pounds in BRL", "20 dollars in euros", "R$200 + 10%"],
    },
    {
      title: "Operators",
      examples: ["8 times 9", "10 plus 5", "20 minus 3"],
    },
    {
      title: "Units",
      examples: ["10 km in miles", "20 ml in tea spoons", "100 celsius in fahrenheit"],
    },
    {
      title: "Dates",
      examples: ["today + 17 days", "today + 3 months"],
    },
    {
      title: "Percentages",
      examples: ["200 + 10%", "20% of $10", "5% on $30 (add)", "10% off 100 (subtract)", "$50 - 5% discount"],
    },
    {
      title: "Totals",
      examples: ["sum / total", "average / avg", "total in USD"],
    },
    {
      title: "Hex / Bin / Oct",
      examples: ["0xFF → 255", "0b1010 → 10", "0o77 → 63"],
    },
    {
      title: "Comments",
      examples: ["// this is a comment", "# this too"],
    },
    {
      title: "Epoch / Timestamp",
      examples: ["now → current timestamp", "fromunix(1446587186)", "tounix(08/21/2026)", "fromunix(1446587186000)"],
    },
  ],
  copyTooltip: "Click to copy",
  copied: "✓ copied",
  total: "Total",
  newTab: "New tab",
  closeTab: "Close tab",
  help: "Help",
  tabDefault: "Tab",
};

const translations: Record<UiLocale, UiStrings> = {
  "pt-br": ptBr,
  en,
};

export function t(locale: UiLocale): UiStrings {
  return translations[locale];
}

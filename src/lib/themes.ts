export type Theme = "dark" | "light";

const darkTheme = {
  "--bg-base": "#1e1e2e",
  "--bg-deep": "#181825",
  "--bg-surface": "#1e1e2e",
  "--bg-elevated": "#313244",
  "--border": "#313244",
  "--border-strong": "#45475a",
  "--text-primary": "#cdd6f4",
  "--text-secondary": "#a6adc8",
  "--text-muted": "#6c7086",
  "--text-faint": "#45475a",
  "--accent": "#cba6f7",
  "--green": "#a6e3a1",
  "--red": "#f38ba8",
  "--yellow": "#f9e2af",
  "--blue": "#89b4fa",
  "--cyan": "#89dceb",
  "--orange": "#fab387",
  "--caret": "#f5e0dc",
};

const lightTheme = {
  "--bg-base": "#eff1f5",
  "--bg-deep": "#e6e9ef",
  "--bg-surface": "#eff1f5",
  "--bg-elevated": "#ccd0da",
  "--border": "#ccd0da",
  "--border-strong": "#bcc0cc",
  "--text-primary": "#4c4f69",
  "--text-secondary": "#5c5f77",
  "--text-muted": "#7c7f93",
  "--text-faint": "#9ca0b0",
  "--accent": "#8839ef",
  "--green": "#40a02b",
  "--red": "#d20f39",
  "--yellow": "#df8e1d",
  "--blue": "#1e66f5",
  "--cyan": "#179299",
  "--orange": "#fe640b",
  "--caret": "#dc8a78",
};

const themes: Record<Theme, Record<string, string>> = {
  dark: darkTheme,
  light: lightTheme,
};

export function applyTheme(theme: Theme) {
  const vars = themes[theme];
  const root = document.documentElement;
  for (const [key, value] of Object.entries(vars)) {
    root.style.setProperty(key, value);
  }
}

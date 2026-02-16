/** Operator! brand palette (from docs/assets/css/main.css) */
export const OPERATOR_BRAND = {
  terracotta: '#E05D44',
  terracottaLight: '#F0796A',
  terracottaDark: '#C24A35',
  cream: '#F2EAC9',
  sage: '#66AA99',
  teal: '#448880',
  deepPine: '#115566',
  cornflower: '#6688AA',
} as const;

/** VS Code CSS variable values extracted from the webview DOM */
export interface VSCodeStyles {
  // Background & foreground
  background: string;
  foreground: string;
  focusBorder: string;

  // Button
  buttonBackground: string;
  buttonForeground: string;
  buttonHoverBackground: string;
  buttonSecondaryBackground: string;
  buttonSecondaryForeground: string;

  // Input
  inputBackground: string;
  inputForeground: string;
  inputBorder: string;
  inputPlaceholderForeground: string;

  // Sidebar / Panel
  sideBarBackground: string;
  sideBarForeground: string;
  sideBarBorder: string;

  // List
  listActiveSelectionBackground: string;
  listActiveSelectionForeground: string;
  listHoverBackground: string;

  // Text
  textLinkForeground: string;
  textLinkActiveForeground: string;
  descriptionForeground: string;

  // Badge
  badgeBackground: string;
  badgeForeground: string;

  // Error / Warning
  errorForeground: string;
  notificationsWarningIconForeground: string;

  // Font
  fontFamily: string;
  fontSize: string;
}

function cssVar(style: CSSStyleDeclaration, name: string, fallback: string): string {
  return style.getPropertyValue(name).trim() || fallback;
}

/** Read ~25 VS Code CSS variables from the webview body */
export function computeStyles(): VSCodeStyles {
  const style = getComputedStyle(document.body);

  return {
    background: cssVar(style, '--vscode-editor-background', '#1e1e1e'),
    foreground: cssVar(style, '--vscode-editor-foreground', '#cccccc'),
    focusBorder: cssVar(style, '--vscode-focusBorder', '#007fd4'),

    buttonBackground: cssVar(style, '--vscode-button-background', '#0e639c'),
    buttonForeground: cssVar(style, '--vscode-button-foreground', '#ffffff'),
    buttonHoverBackground: cssVar(style, '--vscode-button-hoverBackground', '#1177bb'),
    buttonSecondaryBackground: cssVar(style, '--vscode-button-secondaryBackground', '#3a3d41'),
    buttonSecondaryForeground: cssVar(style, '--vscode-button-secondaryForeground', '#cccccc'),

    inputBackground: cssVar(style, '--vscode-input-background', '#3c3c3c'),
    inputForeground: cssVar(style, '--vscode-input-foreground', '#cccccc'),
    inputBorder: cssVar(style, '--vscode-input-border', '#3c3c3c'),
    inputPlaceholderForeground: cssVar(style, '--vscode-input-placeholderForeground', '#a6a6a6'),

    sideBarBackground: cssVar(style, '--vscode-sideBar-background', '#252526'),
    sideBarForeground: cssVar(style, '--vscode-sideBar-foreground', '#cccccc'),
    sideBarBorder: cssVar(style, '--vscode-sideBar-border', '#252526'),

    listActiveSelectionBackground: cssVar(style, '--vscode-list-activeSelectionBackground', '#094771'),
    listActiveSelectionForeground: cssVar(style, '--vscode-list-activeSelectionForeground', '#ffffff'),
    listHoverBackground: cssVar(style, '--vscode-list-hoverBackground', '#2a2d2e'),

    textLinkForeground: cssVar(style, '--vscode-textLink-foreground', '#3794ff'),
    textLinkActiveForeground: cssVar(style, '--vscode-textLink-activeForeground', '#3794ff'),
    descriptionForeground: cssVar(style, '--vscode-descriptionForeground', '#a0a0a0'),

    badgeBackground: cssVar(style, '--vscode-badge-background', '#4d4d4d'),
    badgeForeground: cssVar(style, '--vscode-badge-foreground', '#ffffff'),

    errorForeground: cssVar(style, '--vscode-errorForeground', '#f48771'),
    notificationsWarningIconForeground: cssVar(style, '--vscode-notificationsWarningIcon-foreground', '#cca700'),

    fontFamily: cssVar(style, '--vscode-font-family', "-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif"),
    fontSize: cssVar(style, '--vscode-font-size', '13px'),
  };
}

import { createTheme, type Theme } from '@mui/material/styles';
import { OPERATOR_BRAND, type VSCodeStyles } from './computeStyles';

/** Detect if VS Code is using a dark theme based on body class */
function isDarkTheme(): boolean {
  return document.body.classList.contains('vscode-dark') ||
    document.body.classList.contains('vscode-high-contrast');
}

/** Create an MUI theme that matches the current VS Code color scheme */
export function createVSCodeTheme(styles: VSCodeStyles): Theme {
  const dark = isDarkTheme();

  return createTheme({
    palette: {
      mode: dark ? 'dark' : 'light',
      primary: {
        main: styles.buttonBackground,
        contrastText: styles.buttonForeground,
      },
      secondary: {
        main: styles.buttonSecondaryBackground,
        contrastText: styles.buttonSecondaryForeground,
      },
      background: {
        default: styles.background,
        paper: styles.sideBarBackground,
      },
      text: {
        primary: styles.foreground,
        secondary: styles.descriptionForeground,
      },
      error: {
        main: styles.errorForeground,
      },
      warning: {
        main: styles.notificationsWarningIconForeground,
      },
      divider: styles.sideBarBorder,
    },
    typography: {
      fontFamily: styles.fontFamily,
      fontSize: parseInt(styles.fontSize, 10) || 13,
      h6: {
        fontSize: '1.1rem',
        fontWeight: 600,
      },
      body1: {
        fontSize: '0.9rem',
      },
      body2: {
        fontSize: '0.8rem',
      },
    },
    components: {
      MuiButtonBase: {
        defaultProps: {
          disableRipple: true,
        },
      },
      MuiCssBaseline: {
        styleOverrides: {
          body: {
            backgroundColor: styles.background,
            color: styles.foreground,
          },
        },
      },
      MuiButton: {
        styleOverrides: {
          root: {
            textTransform: 'none',
            borderRadius: 2,
          },
          contained: {
            backgroundColor: styles.buttonBackground,
            color: styles.buttonForeground,
            '&:hover': {
              backgroundColor: styles.buttonHoverBackground,
            },
          },
          outlined: {
            borderColor: styles.inputBorder,
            color: styles.foreground,
            '&:hover': {
              borderColor: styles.focusBorder,
              backgroundColor: 'transparent',
            },
          },
        },
      },
      MuiTextField: {
        defaultProps: {
          margin: 'dense',
        },
      },
      MuiOutlinedInput: {
        styleOverrides: {
          root: {
            backgroundColor: styles.inputBackground,
            color: styles.inputForeground,
            '& .MuiOutlinedInput-notchedOutline': {
              borderColor: styles.inputBorder,
            },
            '&:hover .MuiOutlinedInput-notchedOutline': {
              borderColor: styles.focusBorder,
            },
            '&.Mui-focused .MuiOutlinedInput-notchedOutline': {
              borderColor: styles.focusBorder,
            },
          },
          input: {
            '&::placeholder': {
              color: styles.inputPlaceholderForeground,
              opacity: 1,
            },
          },
        },
      },
      MuiInputLabel: {
        styleOverrides: {
          root: {
            color: styles.descriptionForeground,
            '&.Mui-focused': {
              color: styles.focusBorder,
            },
          },
        },
      },
      MuiLink: {
        styleOverrides: {
          root: {
            color: styles.textLinkForeground,
            '&:hover': {
              color: styles.textLinkActiveForeground,
            },
          },
        },
      },
      MuiListItemButton: {
        styleOverrides: {
          root: {
            '&.Mui-selected': {
              backgroundColor: styles.listActiveSelectionBackground,
              color: styles.listActiveSelectionForeground,
              '&:hover': {
                backgroundColor: styles.listActiveSelectionBackground,
              },
            },
            '&:hover': {
              backgroundColor: styles.listHoverBackground,
            },
          },
        },
      },
      MuiPaper: {
        styleOverrides: {
          root: {
            backgroundImage: 'none',
          },
        },
      },
      MuiSelect: {
        styleOverrides: {
          icon: {
            color: styles.foreground,
          },
        },
      },
      MuiSwitch: {
        styleOverrides: {
          switchBase: {
            '&.Mui-checked': {
              color: OPERATOR_BRAND.terracotta,
              '& + .MuiSwitch-track': {
                backgroundColor: OPERATOR_BRAND.terracotta,
              },
            },
          },
        },
      },
      MuiCard: {
        styleOverrides: {
          root: {
            borderColor: dark
              ? `${OPERATOR_BRAND.terracotta}40`
              : `${OPERATOR_BRAND.terracotta}25`,
          },
        },
      },
      MuiChip: {
        styleOverrides: {
          root: {
            backgroundColor: styles.badgeBackground,
            color: styles.badgeForeground,
          },
        },
      },
    },
  });
}

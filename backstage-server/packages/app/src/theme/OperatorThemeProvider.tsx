/**
 * Operator Theme Provider
 *
 * Provides dynamic theming based on branding configuration from the Operator server.
 * Fetches /api/branding on mount and creates a custom Backstage theme from the colors.
 *
 * Features:
 * - Light/dark mode with system preference detection
 * - Auto-derived dark mode colors from light palette
 * - Customizable component styling (4px border radius, flat buttons)
 * - Configurable via ~/.operator/backstage/branding/theme.json
 */

import { createContext, useContext, useEffect, useState, ReactNode } from 'react';
import {
  UnifiedThemeProvider,
  createUnifiedTheme,
  palettes,
} from '@backstage/theme';

// Theme configuration from server
export interface ThemeConfig {
  appTitle: string;
  orgName: string;
  logoPath?: string;
  mode: 'light' | 'dark' | 'system';
  colors: {
    // Core brand colors
    primary: string;      // Main action color (Terracotta)
    secondary: string;    // Secondary elements (Deep Pine)
    accent: string;       // Highlights, light surfaces (Cream)
    warning: string;      // Alerts
    muted: string;        // Subdued text (Cornflower)
    // Light mode surfaces
    background: string;   // Page background
    surface: string;      // Card/paper background
    text: string;         // Primary text color
    // Navigation scale (4 levels, L1=lightest, L4=darkest)
    navL1: string;        // Nav button default (Sage)
    navL2: string;        // Nav hover (Teal)
    navL3: string;        // Nav selected (Deep Pine)
    navL4: string;        // Nav background/darkest (Midnight)
  };
  components?: {
    borderRadius?: number;  // Default: 4
  };
}

// Default theme config (matches docs/assets/css/main.css)
const defaultThemeConfig: ThemeConfig = {
  appTitle: 'Operator!',
  orgName: 'Operator!',
  logoPath: 'logo.svg',
  mode: 'system',  // Respects OS light/dark preference
  colors: {
    // Core brand (from docs palette)
    primary: '#E05D44',     // Terracotta
    secondary: '#115566',   // Deep Pine
    accent: '#F2EAC9',      // Cream
    warning: '#E05D44',     // Terracotta
    muted: '#6688AA',       // Cornflower
    // Light mode surfaces
    background: '#faf8f5',  // Warm off-white
    surface: '#ffffff',     // Pure white cards
    text: '#115566',        // Deep Pine
    // Navigation green scale (L1=lightest, L4=darkest)
    navL1: '#66AA99',       // Sage - button default
    navL2: '#448880',       // Teal - hover
    navL3: '#115566',       // Deep Pine - selected
    navL4: '#082226',       // Midnight - nav background
  },
  components: {
    borderRadius: 4,        // Subtle rounding
  },
};

// Context for accessing theme config
const ThemeConfigContext = createContext<ThemeConfig | null>(null);

// Hook to access theme configuration
export function useOperatorTheme(): ThemeConfig | null {
  return useContext(ThemeConfigContext);
}

// Hook to detect system dark mode preference
function usePrefersDarkMode(): boolean {
  const [prefersDark, setPrefersDark] = useState(
    () => typeof window !== 'undefined'
      ? window.matchMedia('(prefers-color-scheme: dark)').matches
      : false
  );

  useEffect(() => {
    if (typeof window === 'undefined') {return;}

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handler = (e: MediaQueryListEvent) => setPrefersDark(e.matches);
    mediaQuery.addEventListener('change', handler);
    return () => mediaQuery.removeEventListener('change', handler);
  }, []);

  return prefersDark;
}

// Auto-derive dark mode colors from light palette
interface DarkColors {
  background: string;
  surface: string;
  text: string;
  navButton: string;
  navHover: string;
  navSelected: string;
}

function deriveDarkColors(colors: ThemeConfig['colors']): DarkColors {
  return {
    background: colors.navL4,        // Midnight
    surface: colors.navL3,           // Deep Pine
    text: colors.accent,             // Cream
    navButton: colors.navL2,         // Teal (inverted)
    navHover: colors.navL1,          // Sage (inverted)
    navSelected: colors.accent,      // Cream text on dark
  };
}

// Create a Backstage unified theme from our config
function createOperatorTheme(config: ThemeConfig, isDark: boolean) {
  const basePalette = isDark ? palettes.dark : palettes.light;
  const dark = isDark ? deriveDarkColors(config.colors) : null;
  const radius = config.components?.borderRadius ?? 4;

  return createUnifiedTheme({
    palette: {
      ...basePalette,
      primary: { main: config.colors.primary },
      secondary: { main: config.colors.secondary },
      warning: { main: config.colors.warning },
      background: {
        default: isDark ? dark!.background : config.colors.background,
        paper: isDark ? dark!.surface : config.colors.surface,
      },
      text: {
        primary: isDark ? dark!.text : config.colors.text,
        secondary: config.colors.muted,
      },
      navigation: {
        background: config.colors.navL4,
        indicator: config.colors.primary,
        color: isDark ? dark!.navSelected : config.colors.accent,
        selectedColor: isDark ? dark!.navSelected : '#ffffff',
        navItem: {
          hoverBackground: isDark ? dark!.navHover : config.colors.navL2,
        },
      },
    },
    components: {
      // Flat buttons with subtle rounding
      MuiButton: {
        styleOverrides: {
          root: {
            borderRadius: radius,
            textTransform: 'none' as const,
            boxShadow: 'none',
            '&:hover': { boxShadow: 'none' },
          },
        },
      },
      // Cards with subtle shadow
      MuiCard: {
        styleOverrides: {
          root: {
            borderRadius: radius + 2,
            boxShadow: isDark
              ? '0 1px 3px rgba(0,0,0,0.3)'
              : '0 1px 3px rgba(0,0,0,0.08)',
          },
        },
      },
      // Paper surfaces
      MuiPaper: {
        styleOverrides: {
          root: { borderRadius: radius },
        },
      },
      // Square-ish chips (not pills)
      MuiChip: {
        styleOverrides: {
          root: { borderRadius: radius },
        },
      },
      // Text fields
      MuiTextField: {
        styleOverrides: {
          root: {
            '& .MuiOutlinedInput-root': { borderRadius: radius },
          },
        },
      },
      // Outlined inputs
      MuiOutlinedInput: {
        styleOverrides: {
          root: { borderRadius: radius },
        },
      },
    },
  });
}

// Merge loaded config with defaults, handling partial configs
function mergeConfig(loaded: Partial<ThemeConfig>): ThemeConfig {
  return {
    ...defaultThemeConfig,
    ...loaded,
    colors: {
      ...defaultThemeConfig.colors,
      ...(loaded.colors || {}),
    },
    components: {
      ...defaultThemeConfig.components,
      ...(loaded.components || {}),
    },
  };
}

interface OperatorThemeProviderProps {
  children: ReactNode;
}

export function OperatorThemeProvider({ children }: OperatorThemeProviderProps) {
  const [config, setConfig] = useState<ThemeConfig>(defaultThemeConfig);
  const [loading, setLoading] = useState(true);
  const prefersDarkMode = usePrefersDarkMode();

  useEffect(() => {
    fetch('/api/branding')
      .then(res => res.json())
      .then((data: Partial<ThemeConfig>) => {
        setConfig(mergeConfig(data));
        setLoading(false);
      })
      .catch(() => {
        // Use defaults if fetch fails
        setLoading(false);
      });
  }, []);

  // Show loading state briefly while fetching config
  if (loading) {
    return null;
  }

  // Determine if dark mode should be active
  const isDark = config.mode === 'dark' ||
    (config.mode === 'system' && prefersDarkMode);

  const theme = createOperatorTheme(config, isDark);

  return (
    <ThemeConfigContext.Provider value={config}>
      <UnifiedThemeProvider theme={theme}>
        {children}
      </UnifiedThemeProvider>
    </ThemeConfigContext.Provider>
  );
}

import React, { useEffect, useState, type ReactNode } from 'react';
import { ThemeProvider, type Theme } from '@mui/material/styles';
import CssBaseline from '@mui/material/CssBaseline';
import { computeStyles } from './computeStyles';
import { createVSCodeTheme } from './createVSCodeTheme';

interface ThemeWrapperProps {
  children: ReactNode;
}

export function ThemeWrapper({ children }: ThemeWrapperProps) {
  const [theme, setTheme] = useState<Theme>(() =>
    createVSCodeTheme(computeStyles())
  );

  useEffect(() => {
    // Re-compute theme when VS Code changes theme (body class changes)
    const observer = new MutationObserver(() => {
      setTheme(createVSCodeTheme(computeStyles()));
    });

    observer.observe(document.body, {
      attributes: true,
      attributeFilter: ['class'],
    });

    return () => observer.disconnect();
  }, []);

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      {children}
    </ThemeProvider>
  );
}

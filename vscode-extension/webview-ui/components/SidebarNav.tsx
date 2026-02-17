import React, { useEffect, useState, useCallback } from 'react';
import List from '@mui/material/List';
import ListItemButton from '@mui/material/ListItemButton';
import ListItemText from '@mui/material/ListItemText';
import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import { OperatorBrand } from './OperatorBrand';

export interface NavItem {
  id: string;
  label: string;
  disabled?: boolean;
}

interface SidebarNavProps {
  items: NavItem[];
  scrollContainerRef: React.RefObject<HTMLElement | null>;
}

export function SidebarNav({ items, scrollContainerRef }: SidebarNavProps) {
  const [activeId, setActiveId] = useState<string>(items[0]?.id ?? '');

  const handleClick = useCallback((item: NavItem) => {
    if (item.disabled) { return; }
    const element = document.getElementById(item.id);
    if (element) {
      element.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  }, []);

  useEffect(() => {
    const container = scrollContainerRef.current;
    if (!container) { return; }

    const sectionElements = items
      .filter((item) => !item.disabled)
      .map((item) => document.getElementById(item.id))
      .filter((el): el is HTMLElement => el !== null);

    const observer = new IntersectionObserver(
      (entries) => {
        // Find the topmost visible section
        const visible = entries
          .filter((e) => e.isIntersecting)
          .sort((a, b) => a.boundingClientRect.top - b.boundingClientRect.top);

        if (visible.length > 0) {
          setActiveId(visible[0].target.id);
        }
      },
      {
        root: container,
        rootMargin: '-10% 0px -80% 0px',
        threshold: 0,
      }
    );

    sectionElements.forEach((el) => observer.observe(el));

    return () => observer.disconnect();
  }, [items, scrollContainerRef]);

  return (
    <Box
      sx={{
        width: 200,
        flexShrink: 0,
        borderRight: 1,
        borderColor: 'divider',
        position: 'sticky',
        top: 0,
        alignSelf: 'flex-start',
        py: 1,
      }}
    >
      <Typography
        variant="body2"
        sx={{
          px: 2,
          py: 1,
          fontWeight: 600,
          textTransform: 'uppercase',
          letterSpacing: 0.5,
          color: 'text.secondary',
          fontSize: '0.7rem',
        }}
      >
        <OperatorBrand /> Settings
      </Typography>
      <List dense disablePadding>
        {items.map((item) => (
          <ListItemButton
            key={item.id}
            selected={activeId === item.id && !item.disabled}
            disabled={item.disabled}
            onClick={() => handleClick(item)}
            sx={{ py: 0.5, px: 2 }}
          >
            <ListItemText
              primary={item.label}
              primaryTypographyProps={{ variant: 'body2' }}
            />
          </ListItemButton>
        ))}
      </List>
    </Box>
  );
}

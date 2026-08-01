import React, { useEffect, useState, useCallback } from 'react';
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
    <nav className="op-nav">
      <div className="op-nav-title">
        <OperatorBrand /> Settings
      </div>
      {items.map((item) => (
        <button
          type="button"
          key={item.id}
          className="op-nav-item"
          data-selected={activeId === item.id && !item.disabled ? 'true' : undefined}
          disabled={item.disabled}
          onClick={() => handleClick(item)}
        >
          {item.label}
        </button>
      ))}
    </nav>
  );
}

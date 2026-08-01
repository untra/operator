import React from 'react';

interface SectionHeaderProps {
  id: string;
  title: string;
}

export function SectionHeader({ id, title }: SectionHeaderProps) {
  return (
    <div id={id} className="op-mb-2" style={{ scrollMarginTop: '16px' }}>
      <h2 className="op-h6 op-mb-05">{title}</h2>
      <hr className="op-divider" />
    </div>
  );
}

import React from 'react';
import styles from './Chip.module.css';

export interface ChipProps {
  label: string;
  variant?: 'default' | 'primary' | 'secondary';
  size?: 'small' | 'medium';
}

export const Chip: React.FC<ChipProps> = ({
  label,
  variant = 'default',
  size = 'medium',
}) => {
  const classNames = [
    styles.chip,
    styles[variant],
    size === 'small' ? styles.small : '',
  ]
    .filter(Boolean)
    .join(' ');

  return <span className={classNames}>{label}</span>;
};

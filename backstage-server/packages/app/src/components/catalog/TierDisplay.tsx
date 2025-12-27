/**
 * Tier Display Component
 *
 * Displays the Operator tier with matching icon from the taxonomy.
 */

import React from 'react';
import { makeStyles, Typography } from '@material-ui/core';
import LayersIcon from '@material-ui/icons/Layers';
import LibraryBooksIcon from '@material-ui/icons/LibraryBooks';
import StorageIcon from '@material-ui/icons/Storage';
import BuildIcon from '@material-ui/icons/Build';
import ArchiveIcon from '@material-ui/icons/Archive';
import HelpOutlineIcon from '@material-ui/icons/HelpOutline';

// Tier configuration matching taxonomy.toml
const TIER_CONFIG: Record<string, {
  label: string;
  icon: React.ElementType;
  color: string;
}> = {
  foundation: {
    label: 'Foundation',
    icon: LayersIcon,
    color: '#5C6BC0', // Indigo
  },
  standards: {
    label: 'Standards',
    icon: LibraryBooksIcon,
    color: '#42A5F5', // Blue
  },
  engines: {
    label: 'Engines',
    icon: StorageIcon,
    color: '#66BB6A', // Green
  },
  ecosystem: {
    label: 'Ecosystem',
    icon: BuildIcon,
    color: '#FFA726', // Orange
  },
  noncurrent: {
    label: 'Noncurrent',
    icon: ArchiveIcon,
    color: '#78909C', // Blue Grey
  },
};

const useStyles = makeStyles({
  container: {
    display: 'flex',
    alignItems: 'center',
    gap: 6,
  },
  icon: {
    fontSize: 18,
  },
  label: {
    fontSize: '0.875rem',
    fontWeight: 500,
  },
});

interface TierDisplayProps {
  tier?: string;
  className?: string;
}

export function TierDisplay({ tier, className }: TierDisplayProps) {
  const classes = useStyles();

  if (!tier) {
    return (
      <Typography variant="body2" color="textSecondary">
        —
      </Typography>
    );
  }

  const normalizedTier = tier.toLowerCase();
  const config = TIER_CONFIG[normalizedTier];

  if (!config) {
    return (
      <div className={`${classes.container} ${className || ''}`}>
        <HelpOutlineIcon className={classes.icon} style={{ color: '#9E9E9E' }} />
        <Typography className={classes.label} style={{ color: '#9E9E9E' }}>
          {tier}
        </Typography>
      </div>
    );
  }

  const IconComponent = config.icon;

  return (
    <div className={`${classes.container} ${className || ''}`}>
      <IconComponent className={classes.icon} style={{ color: config.color }} />
      <Typography className={classes.label} style={{ color: config.color }}>
        {config.label}
      </Typography>
    </div>
  );
}

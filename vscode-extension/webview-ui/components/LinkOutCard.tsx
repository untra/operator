import React from 'react';
import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';
import { SectionHeader } from './SectionHeader';

interface LinkOutCardProps {
  id: string;
  title: string;
  description: string;
  onOpen: () => void;
}

export function LinkOutCard({ id, title, description, onOpen }: LinkOutCardProps) {
  return (
    <Box sx={{ mb: 4 }}>
      <SectionHeader id={id} title={title} />
      <Card variant="outlined">
        <CardContent>
          <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 2 }}>
            <Typography variant="body2" color="text.secondary">
              {description}
            </Typography>
            <Button variant="outlined" onClick={onOpen} sx={{ whiteSpace: 'nowrap' }}>
              Open in Operator UI →
            </Button>
          </Box>
        </CardContent>
      </Card>
    </Box>
  );
}

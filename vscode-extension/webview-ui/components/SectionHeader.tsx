import React from 'react';
import Typography from '@mui/material/Typography';
import Divider from '@mui/material/Divider';
import Box from '@mui/material/Box';

interface SectionHeaderProps {
  id: string;
  title: string;
}

export function SectionHeader({ id, title }: SectionHeaderProps) {
  return (
    <Box id={id} sx={{ scrollMarginTop: '16px', mb: 2 }}>
      <Typography variant="h6" sx={{ mb: 0.5 }}>
        {title}
      </Typography>
      <Divider />
    </Box>
  );
}

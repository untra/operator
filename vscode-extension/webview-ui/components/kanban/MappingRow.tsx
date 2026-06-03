import React from 'react';
import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import Link from '@mui/material/Link';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import FormControl from '@mui/material/FormControl';
import type { ExternalIssueTypeSummary, IssueTypeSummary } from '../../types/messages';

interface MappingRowProps {
  external: ExternalIssueTypeSummary;
  operatorTypes: IssueTypeSummary[];
  selectedKey: string | null;
  autoMatchedKey: string | null;
  onSelect: (externalName: string, operatorKey: string | '') => void;
  onViewIssueType: () => void;
}

export function MappingRow({
  external,
  operatorTypes,
  selectedKey,
  autoMatchedKey,
  onSelect,
  onViewIssueType,
}: MappingRowProps) {
  const effectiveKey = selectedKey ?? autoMatchedKey;
  const isOverride = selectedKey !== null && selectedKey !== autoMatchedKey;

  return (
    <Box sx={{ py: 1, borderBottom: '1px solid', borderColor: 'divider' }}>
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
        {/* External type */}
        <Box sx={{ flex: 1, display: 'flex', alignItems: 'center', gap: 1 }}>
          {external.icon_url && (
            <Box
              component="img"
              src={external.icon_url}
              alt=""
              sx={{ width: 16, height: 16 }}
            />
          )}
          <Typography variant="body2" fontWeight={500}>
            {external.name}
          </Typography>
        </Box>

        {/* Arrow */}
        <Typography color="text.secondary" sx={{ px: 1 }}>→</Typography>

        {/* Operator type selector */}
        <Box sx={{ flex: 1 }}>
          <FormControl size="small" fullWidth>
            <Select
              value={effectiveKey ?? ''}
              onChange={(e) => onSelect(external.name, e.target.value as string)}
              displayEmpty
              sx={{
                '& .MuiSelect-select': { py: 0.5 },
                ...(isOverride ? { borderColor: 'info.main' } : {}),
              }}
            >
              <MenuItem value="">
                <em>Unmapped</em>
              </MenuItem>
              {operatorTypes.map((ot) => (
                <MenuItem key={ot.key} value={ot.key}>
                  <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                    <Typography variant="body2">{ot.glyph}</Typography>
                    <Typography variant="body2">{ot.key}</Typography>
                    <Typography variant="caption" color="text.secondary">
                      {ot.name}
                    </Typography>
                  </Box>
                </MenuItem>
              ))}
            </Select>
          </FormControl>
          {autoMatchedKey && !isOverride && (
            <Typography variant="caption" color="text.secondary" sx={{ mt: 0.25, display: 'block' }}>
              auto-matched
            </Typography>
          )}
          {isOverride && (
            <Typography variant="caption" color="info.main" sx={{ mt: 0.25, display: 'block' }}>
              custom override
            </Typography>
          )}
        </Box>
      </Box>

      {/* View the mapped issue type in the hosted Operator UI */}
      {effectiveKey && (
        <Box sx={{ mt: 0.5, ml: 4 }}>
          <Link
            component="button"
            variant="caption"
            onClick={onViewIssueType}
            sx={{ textAlign: 'left' }}
          >
            view issue type →
          </Link>
        </Box>
      )}
    </Box>
  );
}

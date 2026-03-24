import React, { useState } from 'react';
import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import Button from '@mui/material/Button';
import Chip from '@mui/material/Chip';
import CircularProgress from '@mui/material/CircularProgress';
import Alert from '@mui/material/Alert';
import Collapse from '@mui/material/Collapse';
import type { CollectionResponse, IssueTypeSummary } from '../../types/messages';

interface CollectionsSubSectionProps {
  collections: CollectionResponse[];
  collectionsLoading: boolean;
  collectionsError: string | null;
  issueTypes: IssueTypeSummary[];
  onActivateCollection: (name: string) => void;
  onGetCollections: () => void;
  onViewIssueType: (key: string) => void;
  onCreateIssueType: () => void;
}

export function CollectionsSubSection({
  collections,
  collectionsLoading,
  collectionsError,
  issueTypes,
  onActivateCollection,
  onGetCollections,
  onViewIssueType,
  onCreateIssueType,
}: CollectionsSubSectionProps) {
  const [expandedCollection, setExpandedCollection] = useState<string | null>(null);

  if (collectionsLoading) {
    return (
      <Box sx={{ py: 2, display: 'flex', justifyContent: 'center', gap: 1 }}>
        <CircularProgress size={20} />
        <Typography variant="body2" color="text.secondary">Loading collections...</Typography>
      </Box>
    );
  }

  return (
    <Box sx={{ mt: 3 }}>
      <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 1 }}>
        <Typography variant="subtitle1" fontWeight={600}>
          Collections & Issue Types
        </Typography>
        <Box sx={{ display: 'flex', gap: 1 }}>
          <Button size="small" variant="outlined" onClick={onCreateIssueType}>
            Create Issue Type
          </Button>
          <Button size="small" onClick={onGetCollections}>
            Refresh
          </Button>
        </Box>
      </Box>

      {collectionsError && (
        <Alert severity="error" sx={{ mb: 1 }}>{collectionsError}</Alert>
      )}

      {collections.length === 0 ? (
        <Typography variant="body2" color="text.secondary">
          No collections available. Start the Operator API to manage collections.
        </Typography>
      ) : (
        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
          {collections.map((collection) => (
            <Card key={collection.name} variant="outlined">
              <CardContent sx={{ py: 1, '&:last-child': { pb: 1 } }}>
                <Box
                  sx={{ display: 'flex', alignItems: 'center', gap: 1, cursor: 'pointer' }}
                  onClick={() => setExpandedCollection(
                    expandedCollection === collection.name ? null : collection.name
                  )}
                >
                  <Typography variant="body2" fontWeight={600}>
                    {collection.name}
                  </Typography>
                  {collection.is_active && (
                    <Chip label="Active" size="small" color="success" variant="outlined" />
                  )}
                  <Chip
                    label={`${collection.types.length} types`}
                    size="small"
                    variant="outlined"
                  />
                  {collection.description && (
                    <Typography variant="caption" color="text.secondary" sx={{ flex: 1 }}>
                      {collection.description}
                    </Typography>
                  )}
                  {!collection.is_active && (
                    <Button
                      size="small"
                      variant="outlined"
                      onClick={(e) => {
                        e.stopPropagation();
                        onActivateCollection(collection.name);
                      }}
                    >
                      Activate
                    </Button>
                  )}
                </Box>

                <Collapse in={expandedCollection === collection.name}>
                  <Box sx={{ mt: 1, display: 'flex', flexWrap: 'wrap', gap: 0.5 }}>
                    {collection.types.map((typeKey) => {
                      const type = issueTypes.find(t => t.key === typeKey);
                      return (
                        <Chip
                          key={typeKey}
                          label={
                            <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}>
                              {type?.glyph && <span>{type.glyph}</span>}
                              <span>{typeKey}</span>
                              {type && <span style={{ opacity: 0.7 }}>· {type.name}</span>}
                            </Box>
                          }
                          size="small"
                          variant="outlined"
                          onClick={() => onViewIssueType(typeKey)}
                          sx={{ cursor: 'pointer' }}
                        />
                      );
                    })}
                  </Box>
                </Collapse>
              </CardContent>
            </Card>
          ))}
        </Box>
      )}
    </Box>
  );
}

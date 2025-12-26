/**
 * Collections management page component.
 */
import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Content,
  ContentHeader,
  Header,
  HeaderLabel,
  Page,
  Progress,
  SupportButton,
} from '@backstage/core-components';
import {
  Box,
  Button,
  Card,
  CardActions,
  CardContent,
  Chip,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
  Grid,
  Typography,
} from '@material-ui/core';
import { makeStyles } from '@material-ui/core/styles';
import ArrowBackIcon from '@material-ui/icons/ArrowBack';
import CheckCircleIcon from '@material-ui/icons/CheckCircle';
import { Alert } from '@material-ui/lab';
import { useCollections, useActivateCollection } from '../hooks/useCollections';
import type { CollectionResponse } from '../api/types';

const useStyles = makeStyles(theme => ({
  card: {
    height: '100%',
    display: 'flex',
    flexDirection: 'column',
  },
  cardContent: {
    flexGrow: 1,
  },
  activeCard: {
    borderColor: theme.palette.success.main,
    borderWidth: 2,
  },
  activeBadge: {
    backgroundColor: theme.palette.success.main,
    color: theme.palette.success.contrastText,
  },
  typeChip: {
    margin: theme.spacing(0.5),
  },
  typesContainer: {
    marginTop: theme.spacing(2),
  },
}));

/** Collection card component */
function CollectionCard({
  collection,
  onActivate,
  activating,
}: {
  collection: CollectionResponse;
  onActivate: () => void;
  activating: boolean;
}) {
  const classes = useStyles();

  return (
    <Card
      className={`${classes.card} ${collection.is_active ? classes.activeCard : ''}`}
      variant="outlined"
    >
      <CardContent className={classes.cardContent}>
        <Box display="flex" alignItems="center" justifyContent="space-between">
          <Typography variant="h6">{collection.name}</Typography>
          {collection.is_active && (
            <Chip
              icon={<CheckCircleIcon />}
              label="Active"
              size="small"
              className={classes.activeBadge}
            />
          )}
        </Box>

        <Typography variant="body2" color="textSecondary" style={{ marginTop: 8 }}>
          {collection.description}
        </Typography>

        <Box className={classes.typesContainer}>
          <Typography variant="caption" color="textSecondary">
            Issue Types ({collection.types.length}):
          </Typography>
          <Box mt={1}>
            {collection.types.map(type => (
              <Chip
                key={type}
                label={type}
                size="small"
                className={classes.typeChip}
                variant="outlined"
              />
            ))}
          </Box>
        </Box>
      </CardContent>

      <CardActions>
        {!collection.is_active && (
          <Button
            size="small"
            color="primary"
            onClick={onActivate}
            disabled={activating}
          >
            {activating ? 'Activating...' : 'Activate'}
          </Button>
        )}
      </CardActions>
    </Card>
  );
}

/** Main collections page component */
export function CollectionsPage() {
  const navigate = useNavigate();
  const { collections, loading, error, retry } = useCollections();
  const { activateCollection, activating } = useActivateCollection();
  const [confirmDialog, setConfirmDialog] = useState<string | null>(null);
  const [activatingName, setActivatingName] = useState<string | null>(null);

  if (loading) {
    return <Progress />;
  }

  const handleActivate = async (name: string) => {
    setActivatingName(name);
    try {
      await activateCollection(name);
      retry();
    } finally {
      setActivatingName(null);
      setConfirmDialog(null);
    }
  };

  const activeCollection = collections?.find(c => c.is_active);

  return (
    <Page themeId="tool">
      <Header title="Collections" subtitle="Manage issue type collections">
        {activeCollection && (
          <HeaderLabel label="Active" value={activeCollection.name} />
        )}
      </Header>
      <Content>
        <ContentHeader title="">
          <Button startIcon={<ArrowBackIcon />} onClick={() => navigate('..')}>
            Back to Issue Types
          </Button>
          <SupportButton>
            Collections group issue types together. Only one collection can be
            active at a time.
          </SupportButton>
        </ContentHeader>

        {error && (
          <Alert
            severity="error"
            action={<Button onClick={retry}>Retry</Button>}
            style={{ marginBottom: 16 }}
          >
            Failed to load collections: {error.message}
          </Alert>
        )}

        <Grid container spacing={3}>
          {collections?.map(collection => (
            <Grid item xs={12} sm={6} md={4} key={collection.name}>
              <CollectionCard
                collection={collection}
                onActivate={() => setConfirmDialog(collection.name)}
                activating={activating && activatingName === collection.name}
              />
            </Grid>
          ))}
          {collections?.length === 0 && (
            <Grid item xs={12}>
              <Typography variant="body1" color="textSecondary">
                No collections found.
              </Typography>
            </Grid>
          )}
        </Grid>

        {/* Activation confirmation dialog */}
        <Dialog
          open={Boolean(confirmDialog)}
          onClose={() => setConfirmDialog(null)}
        >
          <DialogTitle>Activate Collection</DialogTitle>
          <DialogContent>
            <DialogContentText>
              Are you sure you want to activate the collection "{confirmDialog}"?
              This will deactivate the currently active collection.
            </DialogContentText>
          </DialogContent>
          <DialogActions>
            <Button onClick={() => setConfirmDialog(null)}>Cancel</Button>
            <Button
              onClick={() => confirmDialog && handleActivate(confirmDialog)}
              color="primary"
              disabled={activating}
            >
              {activating ? 'Activating...' : 'Activate'}
            </Button>
          </DialogActions>
        </Dialog>
      </Content>
    </Page>
  );
}

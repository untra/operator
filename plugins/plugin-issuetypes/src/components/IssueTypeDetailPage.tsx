/**
 * Issue Type detail page component.
 */
import React, { useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import {
  Content,
  ContentHeader,
  Header,
  HeaderLabel,
  Page,
  Progress,
} from '@backstage/core-components';
import {
  Box,
  Button,
  Card,
  CardContent,
  Chip,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
  Divider,
  Grid,
  List,
  ListItem,
  ListItemText,
  Paper,
  Typography,
} from '@material-ui/core';
import { makeStyles } from '@material-ui/core/styles';
import ArrowBackIcon from '@material-ui/icons/ArrowBack';
import DeleteIcon from '@material-ui/icons/Delete';
import EditIcon from '@material-ui/icons/Edit';
import { Alert } from '@material-ui/lab';
import { useIssueType, useDeleteIssueType } from '../hooks/useIssueTypes';
import type { StepResponse, FieldResponse } from '../api/types';

const useStyles = makeStyles(theme => ({
  header: {
    marginBottom: theme.spacing(2),
  },
  glyph: {
    fontSize: '4rem',
    fontWeight: 'bold',
    color: theme.palette.primary.main,
  },
  section: {
    marginTop: theme.spacing(3),
  },
  sectionTitle: {
    marginBottom: theme.spacing(2),
  },
  stepCard: {
    marginBottom: theme.spacing(2),
  },
  stepNumber: {
    backgroundColor: theme.palette.primary.main,
    color: theme.palette.primary.contrastText,
    borderRadius: '50%',
    width: 32,
    height: 32,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: theme.spacing(2),
  },
  prompt: {
    backgroundColor: theme.palette.background.default,
    padding: theme.spacing(1),
    borderRadius: theme.shape.borderRadius,
    fontFamily: 'monospace',
    fontSize: '0.875rem',
    maxHeight: 200,
    overflow: 'auto',
  },
  chip: {
    margin: theme.spacing(0.5),
  },
  metadataRow: {
    display: 'flex',
    alignItems: 'center',
    marginBottom: theme.spacing(1),
  },
  metadataLabel: {
    fontWeight: 'bold',
    marginRight: theme.spacing(1),
    minWidth: 120,
  },
  actions: {
    marginTop: theme.spacing(2),
  },
}));

/** Step card component */
function StepCard({ step, index }: { step: StepResponse; index: number }) {
  const classes = useStyles();

  return (
    <Card className={classes.stepCard} variant="outlined">
      <CardContent>
        <Box display="flex" alignItems="flex-start">
          <Box className={classes.stepNumber}>{index + 1}</Box>
          <Box flex={1}>
            <Typography variant="h6">
              {step.display_name || step.name}
            </Typography>
            <Typography variant="caption" color="textSecondary">
              {step.name}
            </Typography>

            <Box mt={1}>
              <Typography variant="subtitle2">Prompt:</Typography>
              <Paper className={classes.prompt} elevation={0}>
                {step.prompt}
              </Paper>
            </Box>

            <Box mt={2}>
              <Chip
                label={`Permission: ${step.permission_mode}`}
                size="small"
                className={classes.chip}
                variant="outlined"
              />
              {step.requires_review && (
                <Chip
                  label="Requires Review"
                  size="small"
                  className={classes.chip}
                  color="secondary"
                />
              )}
              {step.next_step && (
                <Chip
                  label={`Next: ${step.next_step}`}
                  size="small"
                  className={classes.chip}
                  variant="outlined"
                />
              )}
            </Box>

            {step.outputs.length > 0 && (
              <Box mt={1}>
                <Typography variant="caption">Outputs: </Typography>
                {step.outputs.map(output => (
                  <Chip
                    key={output}
                    label={output}
                    size="small"
                    className={classes.chip}
                  />
                ))}
              </Box>
            )}

            {step.allowed_tools.length > 0 && step.allowed_tools[0] !== '*' && (
              <Box mt={1}>
                <Typography variant="caption">Allowed Tools: </Typography>
                {step.allowed_tools.map(tool => (
                  <Chip
                    key={tool}
                    label={tool}
                    size="small"
                    className={classes.chip}
                    variant="outlined"
                  />
                ))}
              </Box>
            )}
          </Box>
        </Box>
      </CardContent>
    </Card>
  );
}

/** Field list component */
function FieldsList({ fields }: { fields: FieldResponse[] }) {
  if (fields.length === 0) {
    return (
      <Typography variant="body2" color="textSecondary">
        No custom fields defined.
      </Typography>
    );
  }

  return (
    <List dense>
      {fields.map(field => (
        <ListItem key={field.name}>
          <ListItemText
            primary={
              <Box display="flex" alignItems="center">
                <Typography variant="body1">{field.name}</Typography>
                <Chip
                  label={field.field_type}
                  size="small"
                  style={{ marginLeft: 8 }}
                />
                {field.required && (
                  <Chip
                    label="required"
                    size="small"
                    color="secondary"
                    style={{ marginLeft: 4 }}
                  />
                )}
              </Box>
            }
            secondary={field.description}
          />
        </ListItem>
      ))}
    </List>
  );
}

/** Main detail page component */
export function IssueTypeDetailPage() {
  const classes = useStyles();
  const navigate = useNavigate();
  const { key } = useParams<{ key: string }>();
  const { issueType, loading, error, retry } = useIssueType(key || '');
  const { deleteIssueType, deleting } = useDeleteIssueType();
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);

  if (loading) {
    return <Progress />;
  }

  if (error) {
    return (
      <Page themeId="tool">
        <Header title="Issue Type" />
        <Content>
          <Alert severity="error" action={<Button onClick={retry}>Retry</Button>}>
            Failed to load issue type: {error.message}
          </Alert>
        </Content>
      </Page>
    );
  }

  if (!issueType) {
    return (
      <Page themeId="tool">
        <Header title="Issue Type" />
        <Content>
          <Alert severity="warning">Issue type not found.</Alert>
        </Content>
      </Page>
    );
  }

  const isBuiltin = issueType.source === 'builtin';

  const handleDelete = async () => {
    await deleteIssueType(issueType.key);
    navigate('..');
  };

  return (
    <Page themeId="tool">
      <Header title={issueType.name} subtitle={`Issue Type: ${issueType.key}`}>
        <HeaderLabel label="Mode" value={issueType.mode} />
        <HeaderLabel label="Source" value={issueType.source} />
      </Header>
      <Content>
        <ContentHeader title="">
          <Button startIcon={<ArrowBackIcon />} onClick={() => navigate('..')}>
            Back
          </Button>
          {!isBuiltin && (
            <>
              <Button
                variant="outlined"
                startIcon={<EditIcon />}
                onClick={() => navigate('edit')}
                style={{ marginLeft: 8 }}
              >
                Edit
              </Button>
              <Button
                variant="outlined"
                color="secondary"
                startIcon={<DeleteIcon />}
                onClick={() => setDeleteDialogOpen(true)}
                style={{ marginLeft: 8 }}
              >
                Delete
              </Button>
            </>
          )}
        </ContentHeader>

        <Grid container spacing={3}>
          <Grid item xs={12} md={4}>
            <Card>
              <CardContent>
                <Box display="flex" alignItems="center" className={classes.header}>
                  <Typography className={classes.glyph}>
                    {issueType.glyph}
                  </Typography>
                </Box>

                <div className={classes.metadataRow}>
                  <Typography className={classes.metadataLabel}>Key:</Typography>
                  <Typography>{issueType.key}</Typography>
                </div>
                <div className={classes.metadataRow}>
                  <Typography className={classes.metadataLabel}>Name:</Typography>
                  <Typography>{issueType.name}</Typography>
                </div>
                <div className={classes.metadataRow}>
                  <Typography className={classes.metadataLabel}>Mode:</Typography>
                  <Chip
                    label={issueType.mode}
                    size="small"
                    color={issueType.mode === 'autonomous' ? 'primary' : 'default'}
                  />
                </div>
                <div className={classes.metadataRow}>
                  <Typography className={classes.metadataLabel}>
                    Branch Prefix:
                  </Typography>
                  <Typography>{issueType.branch_prefix}</Typography>
                </div>
                <div className={classes.metadataRow}>
                  <Typography className={classes.metadataLabel}>
                    Project Required:
                  </Typography>
                  <Typography>
                    {issueType.project_required ? 'Yes' : 'No'}
                  </Typography>
                </div>
                {issueType.color && (
                  <div className={classes.metadataRow}>
                    <Typography className={classes.metadataLabel}>Color:</Typography>
                    <Box
                      style={{
                        backgroundColor: issueType.color,
                        width: 24,
                        height: 24,
                        borderRadius: 4,
                      }}
                    />
                  </div>
                )}

                <Divider style={{ margin: '16px 0' }} />

                <Typography variant="body2">{issueType.description}</Typography>
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} md={8}>
            <Box className={classes.section}>
              <Typography variant="h5" className={classes.sectionTitle}>
                Steps ({issueType.steps.length})
              </Typography>
              {issueType.steps.map((step, index) => (
                <StepCard key={step.name} step={step} index={index} />
              ))}
            </Box>

            <Box className={classes.section}>
              <Typography variant="h5" className={classes.sectionTitle}>
                Fields ({issueType.fields.length})
              </Typography>
              <Card variant="outlined">
                <CardContent>
                  <FieldsList fields={issueType.fields} />
                </CardContent>
              </Card>
            </Box>
          </Grid>
        </Grid>

        {/* Delete confirmation dialog */}
        <Dialog open={deleteDialogOpen} onClose={() => setDeleteDialogOpen(false)}>
          <DialogTitle>Delete Issue Type</DialogTitle>
          <DialogContent>
            <DialogContentText>
              Are you sure you want to delete the issue type "{issueType.name}" (
              {issueType.key})? This action cannot be undone.
            </DialogContentText>
          </DialogContent>
          <DialogActions>
            <Button onClick={() => setDeleteDialogOpen(false)}>Cancel</Button>
            <Button
              onClick={handleDelete}
              color="secondary"
              disabled={deleting}
            >
              {deleting ? 'Deleting...' : 'Delete'}
            </Button>
          </DialogActions>
        </Dialog>
      </Content>
    </Page>
  );
}

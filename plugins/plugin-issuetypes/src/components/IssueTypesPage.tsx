/**
 * Issue Types list page component.
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
  CardActionArea,
  CardContent,
  Chip,
  FormControl,
  Grid,
  InputLabel,
  MenuItem,
  Select,
  Typography,
} from '@material-ui/core';
import { makeStyles } from '@material-ui/core/styles';
import AddIcon from '@material-ui/icons/Add';
import { Alert } from '@material-ui/lab';
import { useIssueTypes } from '../hooks/useIssueTypes';
import type { IssueTypeSummary } from '../api/types';

const useStyles = makeStyles(theme => ({
  card: {
    height: '100%',
    display: 'flex',
    flexDirection: 'column',
  },
  cardContent: {
    flexGrow: 1,
  },
  glyph: {
    fontSize: '2rem',
    fontWeight: 'bold',
    color: theme.palette.primary.main,
    marginBottom: theme.spacing(1),
  },
  modeBadge: {
    marginTop: theme.spacing(1),
  },
  autonomousBadge: {
    backgroundColor: theme.palette.success.main,
    color: theme.palette.success.contrastText,
  },
  pairedBadge: {
    backgroundColor: theme.palette.warning.main,
    color: theme.palette.warning.contrastText,
  },
  builtinChip: {
    marginLeft: theme.spacing(1),
    fontSize: '0.7rem',
  },
  filterBar: {
    marginBottom: theme.spacing(2),
  },
  stepCount: {
    color: theme.palette.text.secondary,
    marginTop: theme.spacing(1),
  },
}));

/** Card component for a single issue type */
function IssueTypeCard({ issueType }: { issueType: IssueTypeSummary }) {
  const classes = useStyles();
  const navigate = useNavigate();

  const isBuiltin = issueType.source === 'builtin';
  const isAutonomous = issueType.mode === 'autonomous';

  return (
    <Card className={classes.card}>
      <CardActionArea onClick={() => navigate(issueType.key)}>
        <CardContent className={classes.cardContent}>
          <Typography className={classes.glyph}>{issueType.glyph}</Typography>
          <Box display="flex" alignItems="center">
            <Typography variant="h6">{issueType.key}</Typography>
            {isBuiltin && (
              <Chip
                label="builtin"
                size="small"
                className={classes.builtinChip}
                variant="outlined"
              />
            )}
          </Box>
          <Typography variant="subtitle1" color="textSecondary">
            {issueType.name}
          </Typography>
          <Typography variant="body2" color="textSecondary">
            {issueType.description.length > 100
              ? `${issueType.description.substring(0, 100)}...`
              : issueType.description}
          </Typography>
          <Box display="flex" alignItems="center" className={classes.modeBadge}>
            <Chip
              label={issueType.mode}
              size="small"
              className={isAutonomous ? classes.autonomousBadge : classes.pairedBadge}
            />
          </Box>
          <Typography variant="caption" className={classes.stepCount}>
            {issueType.step_count} step{issueType.step_count !== 1 ? 's' : ''}
          </Typography>
        </CardContent>
      </CardActionArea>
    </Card>
  );
}

/** Main issue types page component */
export function IssueTypesPage() {
  const classes = useStyles();
  const navigate = useNavigate();
  const { issueTypes, loading, error, retry } = useIssueTypes();
  const [sourceFilter, setSourceFilter] = useState<string>('all');

  if (loading) {
    return <Progress />;
  }

  const filteredTypes = issueTypes?.filter(t => {
    if (sourceFilter === 'all') return true;
    if (sourceFilter === 'builtin') return t.source === 'builtin';
    if (sourceFilter === 'user') return t.source !== 'builtin';
    return true;
  });

  return (
    <Page themeId="tool">
      <Header title="Issue Types" subtitle="Manage issue type templates">
        <HeaderLabel label="Owner" value="Operator" />
      </Header>
      <Content>
        <ContentHeader title="">
          <Button
            variant="contained"
            color="primary"
            startIcon={<AddIcon />}
            onClick={() => navigate('new')}
          >
            Create Issue Type
          </Button>
          <SupportButton>
            Issue types define the workflow steps for different ticket types.
          </SupportButton>
        </ContentHeader>

        {error && (
          <Alert severity="error" action={<Button onClick={retry}>Retry</Button>}>
            Failed to load issue types: {error.message}
          </Alert>
        )}

        <Box className={classes.filterBar}>
          <FormControl variant="outlined" size="small" style={{ minWidth: 150 }}>
            <InputLabel margin='dense'>Source</InputLabel>
            <Select
              value={sourceFilter}
              onChange={e => setSourceFilter(e.target.value as string)}
              label="Source"
            >
              <MenuItem value="all">All</MenuItem>
              <MenuItem value="builtin">Builtin</MenuItem>
              <MenuItem value="user">User-defined</MenuItem>
            </Select>
          </FormControl>
        </Box>

        <Grid container spacing={3}>
          {filteredTypes?.map(issueType => (
            <Grid item xs={12} sm={6} md={4} lg={3} key={issueType.key}>
              <IssueTypeCard issueType={issueType} />
            </Grid>
          ))}
          {filteredTypes?.length === 0 && (
            <Grid item xs={12}>
              <Typography variant="body1" color="textSecondary">
                No issue types found.
              </Typography>
            </Grid>
          )}
        </Grid>
      </Content>
    </Page>
  );
}

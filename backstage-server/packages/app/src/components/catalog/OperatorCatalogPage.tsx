/**
 * Operator Catalog Page
 *
 * Custom catalog page that defaults to an Operator-focused view (hiding owner/system)
 * with a toggle to switch to the full Backstage enterprise view.
 */

import { useMemo } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import {
  Content,
  ContentHeader,
  PageWithHeader,
  SupportButton,
} from '@backstage/core-components';
import { CatalogTable } from '@backstage/plugin-catalog';
import {
  CatalogFilterLayout,
  EntityListProvider,
  EntityKindPicker,
  EntityTagPicker,
  EntityOwnerPicker,
  EntityLifecyclePicker,
  EntityTypePicker,
} from '@backstage/plugin-catalog-react';
import {
  Button,
  ButtonGroup,
  makeStyles,
  Typography,
  Box,
} from '@material-ui/core';
import ViewModuleIcon from '@material-ui/icons/ViewModule';
import BusinessIcon from '@material-ui/icons/Business';
import { getOperatorColumns, getBackstageColumns } from './columns';

const useStyles = makeStyles((theme) => ({
  viewToggle: {
    marginLeft: 'auto',
  },
  toggleButton: {
    textTransform: 'none',
    padding: '6px 16px',
  },
  activeButton: {
    backgroundColor: theme.palette.primary.main,
    color: theme.palette.primary.contrastText,
    '&:hover': {
      backgroundColor: theme.palette.primary.dark,
    },
  },
  headerRow: {
    display: 'flex',
    alignItems: 'center',
    gap: theme.spacing(2),
    marginBottom: theme.spacing(2),
  },
  filterSection: {
    display: 'flex',
    flexDirection: 'column',
    gap: theme.spacing(2),
  },
}));

type ViewMode = 'operator' | 'backstage';

function useViewMode(): [ViewMode, (mode: ViewMode) => void] {
  const location = useLocation();
  const navigate = useNavigate();

  const viewMode = useMemo(() => {
    const params = new URLSearchParams(location.search);
    const view = params.get('view');
    return view === 'backstage' ? 'backstage' : 'operator';
  }, [location.search]);

  const setViewMode = (mode: ViewMode) => {
    const params = new URLSearchParams(location.search);
    if (mode === 'backstage') {
      params.set('view', 'backstage');
    } else {
      params.delete('view');
    }
    const newSearch = params.toString();
    navigate({
      pathname: location.pathname,
      search: newSearch ? `?${newSearch}` : '',
    }, { replace: true });
  };

  return [viewMode, setViewMode];
}

function ViewModeToggle({
  viewMode,
  onViewModeChange,
}: {
  viewMode: ViewMode;
  onViewModeChange: (mode: ViewMode) => void;
}) {
  const classes = useStyles();

  return (
    <Box className={classes.viewToggle}>
      <ButtonGroup size="small" variant="outlined">
        <Button
          className={`${classes.toggleButton} ${viewMode === 'operator' ? classes.activeButton : ''}`}
          onClick={() => onViewModeChange('operator')}
          startIcon={<ViewModuleIcon />}
        >
          Operator
        </Button>
        <Button
          className={`${classes.toggleButton} ${viewMode === 'backstage' ? classes.activeButton : ''}`}
          onClick={() => onViewModeChange('backstage')}
          startIcon={<BusinessIcon />}
        >
          Backstage
        </Button>
      </ButtonGroup>
    </Box>
  );
}

function OperatorFilters() {
  return (
    <>
      <EntityKindPicker />
      <EntityTypePicker />
      <EntityTagPicker />
    </>
  );
}

function BackstageFilters() {
  return (
    <>
      <EntityKindPicker />
      <EntityTypePicker />
      <EntityOwnerPicker />
      <EntityLifecyclePicker />
      <EntityTagPicker />
    </>
  );
}

function CatalogTableView({ viewMode }: { viewMode: ViewMode }) {
  const columns = useMemo(() => {
    return viewMode === 'operator' ? getOperatorColumns() : getBackstageColumns();
  }, [viewMode]);

  return (
    <CatalogTable
      columns={columns}
    />
  );
}

function CatalogContent({ viewMode }: { viewMode: ViewMode }) {
  return (
    <CatalogFilterLayout>
      <CatalogFilterLayout.Filters>
        {viewMode === 'operator' ? <OperatorFilters /> : <BackstageFilters />}
      </CatalogFilterLayout.Filters>
      <CatalogFilterLayout.Content>
        <CatalogTableView viewMode={viewMode} />
      </CatalogFilterLayout.Content>
    </CatalogFilterLayout>
  );
}

export function OperatorCatalogPage() {
  const classes = useStyles();
  const [viewMode, setViewMode] = useViewMode();

  return (
    <PageWithHeader title="Repositories" themeId="home">
      <Content>
        <Box className={classes.headerRow}>
          <ContentHeader title="">
            <SupportButton>
              {viewMode === 'operator' ? (
                <Typography>
                  Viewing repositories organized by Operator taxonomy.
                  Switch to Backstage view for owner and system information.
                </Typography>
              ) : (
                <Typography>
                  Viewing standard Backstage catalog with owner and system columns.
                  Switch to Operator view for tier-based organization.
                </Typography>
              )}
            </SupportButton>
          </ContentHeader>
          <ViewModeToggle viewMode={viewMode} onViewModeChange={setViewMode} />
        </Box>
        <EntityListProvider>
          <CatalogContent viewMode={viewMode} />
        </EntityListProvider>
      </Content>
    </PageWithHeader>
  );
}

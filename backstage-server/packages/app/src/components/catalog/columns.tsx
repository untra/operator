/**
 * Catalog Table Column Definitions
 *
 * Defines columns for Operator mode and Backstage mode views.
 * Uses CatalogTableColumnsFunc pattern for compatibility with CatalogTable.
 */

import { Link, OverflowTooltip, TableColumn } from '@backstage/core-components';
import { Chip, makeStyles, Tooltip } from '@material-ui/core';
import { Entity } from '@backstage/catalog-model';
import { CatalogTableRow } from '@backstage/plugin-catalog';
import { ApiTypeBadge, isApiEntity } from './ApiTypeBadge';
import { TierDisplay } from './TierDisplay';

const useStyles = makeStyles({
  nameCell: {
    display: 'flex',
    alignItems: 'center',
    gap: 4,
  },
  tagChip: {
    height: 20,
    fontSize: '0.7rem',
    margin: 2,
  },
  tagsContainer: {
    display: 'flex',
    flexWrap: 'wrap',
    gap: 2,
  },
});

// Format Operator kind from spec.type (e.g., "api-gateway" -> "API Gateway")
function formatOperatorKind(specType?: string): string {
  if (!specType) return '—';
  return specType
    .split('-')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
}

// ============================================================================
// Operator Mode Columns
// ============================================================================

function NameCellOperator({ entity }: { entity: Entity }) {
  const classes = useStyles();
  const title = entity.metadata.title || entity.metadata.name;
  const namespace = entity.metadata.namespace || 'default';
  const kind = entity.kind.toLowerCase();
  const isApi = isApiEntity(entity);
  const apiType = entity.spec?.type as string | undefined;

  return (
    <div className={classes.nameCell}>
      <Link to={`/catalog/${namespace}/${kind}/${entity.metadata.name}`}>
        {title}
      </Link>
      {isApi && <ApiTypeBadge apiType={apiType} />}
    </div>
  );
}

function TagsCell({ entity }: { entity: Entity }) {
  const classes = useStyles();
  const tags = entity.metadata.tags || [];

  if (tags.length === 0) {
    return <span>—</span>;
  }

  const displayTags = tags.slice(0, 3);
  const remainingCount = tags.length - 3;

  return (
    <div className={classes.tagsContainer}>
      {displayTags.map(tag => (
        <Chip
          key={tag}
          label={tag}
          size="small"
          className={classes.tagChip}
          variant="outlined"
        />
      ))}
      {remainingCount > 0 && (
        <Tooltip title={tags.slice(3).join(', ')}>
          <Chip
            label={`+${remainingCount}`}
            size="small"
            className={classes.tagChip}
            variant="outlined"
          />
        </Tooltip>
      )}
    </div>
  );
}

export function getOperatorColumns(): TableColumn<CatalogTableRow>[] {
  return [
    {
      title: 'Name',
      field: 'resolved.name',
      highlight: true,
      render: (row: CatalogTableRow) => <NameCellOperator entity={row.entity} />,
    },
    {
      title: 'Kind',
      field: 'entity.spec.type',
      render: (row: CatalogTableRow) => {
        const specType = row.entity.spec?.type as string | undefined;
        return (
          <span style={{ textTransform: 'capitalize' }}>
            {formatOperatorKind(specType)}
          </span>
        );
      },
    },
    {
      title: 'Tier',
      field: 'entity.metadata.labels.operator-tier',
      render: (row: CatalogTableRow) => {
        const tier = row.entity.metadata.labels?.['operator-tier'];
        return <TierDisplay tier={tier} />;
      },
    },
    {
      title: 'Description',
      field: 'entity.metadata.description',
      render: (row: CatalogTableRow) => (
        <OverflowTooltip text={row.entity.metadata.description || '—'} />
      ),
    },
    {
      title: 'Tags',
      field: 'entity.metadata.tags',
      render: (row: CatalogTableRow) => <TagsCell entity={row.entity} />,
    },
  ];
}

// ============================================================================
// Backstage Mode Columns
// ============================================================================

function NameCellBackstage({ entity }: { entity: Entity }) {
  const namespace = entity.metadata.namespace || 'default';
  const kind = entity.kind.toLowerCase();
  const title = entity.metadata.title || entity.metadata.name;

  return (
    <Link to={`/catalog/${namespace}/${kind}/${entity.metadata.name}`}>
      {title}
    </Link>
  );
}

function OwnerCell({ entity }: { entity: Entity }) {
  const owner = entity.spec?.owner as string | undefined;
  if (!owner) return <span>—</span>;

  // Display owner as text - refs may not be fully qualified in local-file mode
  return <span>{owner}</span>;
}

function SystemCell({ entity }: { entity: Entity }) {
  const system = entity.spec?.system as string | undefined;
  if (!system) return <span>—</span>;

  // Display system as text - refs may not be fully qualified in local-file mode
  return <span>{system}</span>;
}

function LifecycleCell({ entity }: { entity: Entity }) {
  const lifecycle = entity.spec?.lifecycle as string | undefined;
  if (!lifecycle) return <span>—</span>;

  const colors: Record<string, string> = {
    production: '#4CAF50',
    experimental: '#FF9800',
    deprecated: '#F44336',
  };

  const color = colors[lifecycle.toLowerCase()] || '#9E9E9E';

  return (
    <Chip
      label={lifecycle}
      size="small"
      style={{
        backgroundColor: color,
        color: '#fff',
        height: 20,
        fontSize: '0.7rem',
        textTransform: 'capitalize',
      }}
    />
  );
}

export function getBackstageColumns(): TableColumn<CatalogTableRow>[] {
  return [
    {
      title: 'Name',
      field: 'resolved.name',
      highlight: true,
      render: (row: CatalogTableRow) => <NameCellBackstage entity={row.entity} />,
    },
    {
      title: 'Kind',
      field: 'entity.kind',
      render: (row: CatalogTableRow) => (
        <span style={{ textTransform: 'capitalize' }}>{row.entity.kind}</span>
      ),
    },
    {
      title: 'Owner',
      field: 'entity.spec.owner',
      render: (row: CatalogTableRow) => <OwnerCell entity={row.entity} />,
    },
    {
      title: 'System',
      field: 'entity.spec.system',
      render: (row: CatalogTableRow) => <SystemCell entity={row.entity} />,
    },
    {
      title: 'Lifecycle',
      field: 'entity.spec.lifecycle',
      render: (row: CatalogTableRow) => <LifecycleCell entity={row.entity} />,
    },
    {
      title: 'Type',
      field: 'entity.spec.type',
      render: (row: CatalogTableRow) => {
        const specType = row.entity.spec?.type as string | undefined;
        return <span>{specType || '—'}</span>;
      },
    },
  ];
}

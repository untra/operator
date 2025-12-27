/**
 * Entity Page Component
 *
 * Displays detailed information about a catalog entity.
 */

import React from 'react';
import { Grid } from '@material-ui/core';
import {
  EntityAboutCard,
  EntityHasSubcomponentsCard,
  EntityLinksCard,
  EntitySwitch,
  EntityOrphanWarning,
  EntityProcessingErrorsPanel,
  isKind,
} from '@backstage/plugin-catalog';
import {
  EntityLayout,
} from '@backstage/plugin-catalog';

const entityWarningContent = (
  <>
    <EntitySwitch>
      <EntitySwitch.Case if={e => Boolean(e.metadata.annotations?.['backstage.io/orphan'])}>
        <Grid item xs={12}>
          <EntityOrphanWarning />
        </Grid>
      </EntitySwitch.Case>
    </EntitySwitch>
    <EntitySwitch>
      <EntitySwitch.Case if={e => Boolean(e.metadata.annotations?.['backstage.io/processing-errors'])}>
        <Grid item xs={12}>
          <EntityProcessingErrorsPanel />
        </Grid>
      </EntitySwitch.Case>
    </EntitySwitch>
  </>
);

const overviewContent = (
  <Grid container spacing={3} alignItems="stretch">
    {entityWarningContent}
    <Grid item md={6}>
      <EntityAboutCard variant="gridItem" />
    </Grid>
    <Grid item md={6}>
      <EntityLinksCard />
    </Grid>
    <Grid item md={12}>
      <EntityHasSubcomponentsCard variant="gridItem" />
    </Grid>
  </Grid>
);

const componentPage = (
  <EntityLayout>
    <EntityLayout.Route path="/" title="Overview">
      {overviewContent}
    </EntityLayout.Route>
  </EntityLayout>
);

const apiPage = (
  <EntityLayout>
    <EntityLayout.Route path="/" title="Overview">
      {overviewContent}
    </EntityLayout.Route>
  </EntityLayout>
);

const systemPage = (
  <EntityLayout>
    <EntityLayout.Route path="/" title="Overview">
      {overviewContent}
    </EntityLayout.Route>
  </EntityLayout>
);

const domainPage = (
  <EntityLayout>
    <EntityLayout.Route path="/" title="Overview">
      {overviewContent}
    </EntityLayout.Route>
  </EntityLayout>
);

const defaultPage = (
  <EntityLayout>
    <EntityLayout.Route path="/" title="Overview">
      {overviewContent}
    </EntityLayout.Route>
  </EntityLayout>
);

export function entityPage() {
  return (
    <EntitySwitch>
      <EntitySwitch.Case if={isKind('component')} children={componentPage} />
      <EntitySwitch.Case if={isKind('api')} children={apiPage} />
      <EntitySwitch.Case if={isKind('system')} children={systemPage} />
      <EntitySwitch.Case if={isKind('domain')} children={domainPage} />
      <EntitySwitch.Case>{defaultPage}</EntitySwitch.Case>
    </EntitySwitch>
  );
}

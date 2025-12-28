/**
 * Homepage Extensions
 *
 * Extension-based homepage for the new Backstage frontend system.
 * Provides the homepage page extension with widget input slots.
 */

import React from 'react';
import {
  PageBlueprint,
  createRouteRef,
  createExtension,
  coreExtensionData,
  createExtensionInput,
} from '@backstage/frontend-plugin-api';

// Route reference for the homepage
export const homeRouteRef = createRouteRef();

/**
 * Homepage extension using PageBlueprint with widget inputs.
 *
 * This allows other extensions to attach widgets to the homepage
 * by targeting the 'widgets' input.
 */
export const homePageExtension = PageBlueprint.makeWithOverrides({
  name: 'operator-home',
  inputs: {
    widgets: createExtensionInput([coreExtensionData.reactElement], {
      singleton: false,
    }),
  },
  factory(originalFactory, { inputs }) {
    return originalFactory({
      path: '/',
      routeRef: homeRouteRef,
      loader: async () => {
        const { OperatorHomePage } = await import(
          '../components/home/OperatorHomePage'
        );

        // Extract widget elements from inputs
        const widgetElements = inputs.widgets?.map((widget, index) => {
          const element = widget.get(coreExtensionData.reactElement);
          return <React.Fragment key={index}>{element}</React.Fragment>;
        });

        return <OperatorHomePage widgets={widgetElements} />;
      },
    });
  },
});

/**
 * Queue Status Widget Extension
 *
 * Displays ticket queue status on the homepage.
 */
export const queueStatusWidgetExtension = createExtension({
  name: 'queue-status-widget',
  attachTo: { id: 'page:app/operator-home', input: 'widgets' },
  output: [coreExtensionData.reactElement],
  factory() {
    const LazyQueueStatus = React.lazy(() =>
      import('../components/home/widgets/QueueStatusCard').then(m => ({
        default: m.QueueStatusCard,
      }))
    );

    return [
      coreExtensionData.reactElement(
        <React.Suspense fallback={null}>
          <LazyQueueStatus />
        </React.Suspense>
      ),
    ];
  },
});

/**
 * Active Agents Widget Extension
 *
 * Displays running Claude Code agents on the homepage.
 */
export const activeAgentsWidgetExtension = createExtension({
  name: 'active-agents-widget',
  attachTo: { id: 'page:app/operator-home', input: 'widgets' },
  output: [coreExtensionData.reactElement],
  factory() {
    const LazyActiveAgents = React.lazy(() =>
      import('../components/home/widgets/ActiveAgentsCard').then(m => ({
        default: m.ActiveAgentsCard,
      }))
    );

    return [
      coreExtensionData.reactElement(
        <React.Suspense fallback={null}>
          <LazyActiveAgents />
        </React.Suspense>
      ),
    ];
  },
});

/**
 * Issue Types Widget Extension
 *
 * Displays quick access to issue types management on the homepage.
 */
export const issueTypesWidgetExtension = createExtension({
  name: 'issue-types-widget',
  attachTo: { id: 'page:app/operator-home', input: 'widgets' },
  output: [coreExtensionData.reactElement],
  factory() {
    const LazyIssueTypes = React.lazy(() =>
      import('../components/home/widgets/IssueTypesCard').then(m => ({
        default: m.IssueTypesCard,
      }))
    );

    return [
      coreExtensionData.reactElement(
        <React.Suspense fallback={null}>
          <LazyIssueTypes />
        </React.Suspense>
      ),
    ];
  },
});

// Export all homepage-related extensions
export const homepageExtensions = [
  homePageExtension,
  queueStatusWidgetExtension,
  activeAgentsWidgetExtension,
  issueTypesWidgetExtension,
];

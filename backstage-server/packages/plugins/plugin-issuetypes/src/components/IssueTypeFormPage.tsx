/**
 * Issue Type Form Page
 *
 * Create or edit an issue type.
 */

import React from 'react';
import { useParams } from 'react-router-dom';
import {
  Content,
  ContentHeader,
  Header,
  Page,
  InfoCard,
} from '@backstage/core-components';
import styles from './IssueTypeFormPage.module.css';

export const IssueTypeFormPage = () => {
  const { key } = useParams<{ key?: string }>();
  const isEditing = Boolean(key);

  return (
    <Page themeId="tool">
      <Header
        title={isEditing ? `Edit ${key}` : 'New Issue Type'}
        subtitle="Configure issue type settings"
      />
      <Content>
        <ContentHeader title={isEditing ? 'Edit Issue Type' : 'Create Issue Type'} />
        <InfoCard title="Form">
          <p className={styles.body1}>
            Issue type form coming soon. This will allow creating and editing
            custom issue types that can be used with Operator.
          </p>
          <p className={styles.body2}>
            For now, issue types can be configured via JSON files in the
            .tickets/templates/ directory.
          </p>
        </InfoCard>
      </Content>
    </Page>
  );
};

/**
 * Issue Type Detail Page
 *
 * Displays detailed information about a specific issue type.
 */

import React, { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import {
  Content,
  ContentHeader,
  Header,
  Page,
  InfoCard,
} from '@backstage/core-components';
import { useApi, configApiRef } from '@backstage/core-plugin-api';
import { Chip } from './ui';
import styles from './IssueTypeDetailPage.module.css';

interface IssueType {
  key: string;
  name: string;
  glyph: string;
  description: string;
  mode: 'autonomous' | 'paired';
  source: 'builtin' | 'user' | 'imported';
  steps?: string[];
  fields?: Record<string, unknown>;
}

export const IssueTypeDetailPage = () => {
  const { key } = useParams<{ key: string }>();
  const configApi = useApi(configApiRef);
  const [issueType, setIssueType] = useState<IssueType | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchIssueType = async () => {
      try {
        const backendUrl = configApi.getString('backend.baseUrl');
        const response = await fetch(`${backendUrl}/api/proxy/operator/api/issuetypes/${key}`);
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        const data = await response.json();
        setIssueType(data);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch issue type');
      } finally {
        setLoading(false);
      }
    };

    if (key) {
      fetchIssueType();
    }
  }, [configApi, key]);

  if (loading) {
    return <Page themeId="tool"><Content>Loading...</Content></Page>;
  }

  if (error || !issueType) {
    return (
      <Page themeId="tool">
        <Content>
          <p className={styles.error}>{error || 'Issue type not found'}</p>
        </Content>
      </Page>
    );
  }

  return (
    <Page themeId="tool">
      <Header title={`${issueType.glyph} ${issueType.name}`} subtitle={issueType.key} />
      <Content>
        <ContentHeader title="Issue Type Details" />
        <div className={styles.grid}>
          <div className={styles.gridItemHalf}>
            <InfoCard title="Overview">
              <p className={styles.body1}>
                {issueType.description}
              </p>
              <h4 className={styles.subtitle}>Mode</h4>
              <Chip
                label={issueType.mode}
                variant={issueType.mode === 'autonomous' ? 'primary' : 'secondary'}
                size="small"
              />
              <h4 className={styles.subtitleSpaced}>Source</h4>
              <Chip label={issueType.source} size="small" />
            </InfoCard>
          </div>
          {issueType.steps && issueType.steps.length > 0 && (
            <div className={styles.gridItemHalf}>
              <InfoCard title="Workflow Steps">
                <ol>
                  {issueType.steps.map((step, index) => (
                    <li key={index}>{step}</li>
                  ))}
                </ol>
              </InfoCard>
            </div>
          )}
        </div>
      </Content>
    </Page>
  );
};

/**
 * Search Page Component
 *
 * Full-page search interface for the catalog.
 * Uses BUI components for layout.
 */

import { Content, Header, Page } from '@backstage/core-components';
import { Flex, Card, CardBody } from '@backstage/ui';
import {
  SearchBar,
  SearchResult,
  SearchResultPager,
  DefaultResultListItem,
} from '@backstage/plugin-search-react';
import { CatalogSearchResultListItem } from '@backstage/plugin-catalog';
import './SearchPage.css';

export function SearchPage() {
  return (
    <Page themeId="home">
      <Header title="Search" subtitle="Find among repositories" />
      <Content>
        <Flex direction="column" gap="4" p="4">
          <Card className="search-bar-card">
            <CardBody>
              <SearchBar />
            </CardBody>
          </Card>
          <div className="search-results">
            <SearchResult>
              <CatalogSearchResultListItem />
              <DefaultResultListItem />
            </SearchResult>
            <SearchResultPager />
          </div>
        </Flex>
      </Content>
    </Page>
  );
}

export default SearchPage;

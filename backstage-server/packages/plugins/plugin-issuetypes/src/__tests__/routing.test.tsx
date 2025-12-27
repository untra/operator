/**
 * Routing Tests
 *
 * Tests that verify route configuration and component exports for navigation.
 * Uses bun:test for smoke-style testing.
 *
 * These tests verify that:
 * 1. Components are properly exported from the plugin
 * 2. Navigation patterns use relative paths (for flat routes)
 * 3. The App.tsx uses flat routes (not nested)
 */

import { describe, test, expect } from 'bun:test';
import * as fs from 'fs';
import * as path from 'path';

describe('IssueTypes Route Configuration', () => {
  test('App.tsx uses flat routes for issuetypes', () => {
    // Read the App.tsx file to verify routes are flat
    const appPath = path.resolve(
      __dirname,
      '../../../../app/src/App.tsx',
    );
    const content = fs.readFileSync(appPath, 'utf-8');

    // Should have flat routes (each route is independent, not nested)
    expect(content).toContain('path="/issuetypes"');
    expect(content).toContain('path="/issuetypes/new"');
    expect(content).toContain('path="/issuetypes/collections"');
    expect(content).toContain('path="/issuetypes/:key"');
    expect(content).toContain('path="/issuetypes/:key/edit"');

    // Each route should use its own element prop (not nested children)
    expect(content).toContain('element={<IssueTypesPage />}');
    expect(content).toContain('element={<IssueTypeFormPage />}');
    expect(content).toContain('element={<CollectionsPage />}');
    expect(content).toContain('element={<IssueTypeDetailPage />}');
  });

  test('plugin exports all page components', async () => {
    // Read the plugin index.ts to verify exports
    const indexPath = path.resolve(__dirname, '../index.ts');
    const content = fs.readFileSync(indexPath, 'utf-8');

    expect(content).toContain('IssueTypesPage');
    expect(content).toContain('IssueTypeDetailPage');
    expect(content).toContain('IssueTypeFormPage');
    expect(content).toContain('CollectionsPage');
  });
});

describe('Navigation Patterns', () => {
  test('IssueTypesPage uses relative links for navigation', () => {
    const componentPath = path.resolve(
      __dirname,
      '../components/IssueTypesPage.tsx',
    );
    const content = fs.readFileSync(componentPath, 'utf-8');

    // Check that the Create Issue Type button uses relative "new" path
    expect(content).toContain('to="new"');

    // Check that Collections button uses relative "collections" path
    expect(content).toContain('to="collections"');

    // Check that row links use relative paths (just the key)
    expect(content).toContain('to={row.key}');
  });

  test('IssueTypeFormPage uses navigate for programmatic navigation', () => {
    const componentPath = path.resolve(
      __dirname,
      '../components/IssueTypeFormPage.tsx',
    );
    const content = fs.readFileSync(componentPath, 'utf-8');

    // Uses useNavigate hook
    expect(content).toContain('useNavigate');

    // Navigates back with relative path
    expect(content).toContain("navigate('..')");
  });

  test('IssueTypeDetailPage exists and handles key param', () => {
    const componentPath = path.resolve(
      __dirname,
      '../components/IssueTypeDetailPage.tsx',
    );
    const content = fs.readFileSync(componentPath, 'utf-8');

    // Should use useParams to get the key
    expect(content).toContain('useParams');
    expect(content).toContain('key');
  });
});

describe('Route Priority (flat routes ensure specificity)', () => {
  test('/issuetypes/new is more specific than /issuetypes/:key', () => {
    const appPath = path.resolve(
      __dirname,
      '../../../../app/src/App.tsx',
    );
    const content = fs.readFileSync(appPath, 'utf-8');

    // Get positions of routes
    const newRoutePos = content.indexOf('path="/issuetypes/new"');
    const keyRoutePos = content.indexOf('path="/issuetypes/:key"');

    // /issuetypes/new should come before /issuetypes/:key for proper matching
    expect(newRoutePos).toBeLessThan(keyRoutePos);
  });

  test('/issuetypes/collections is more specific than /issuetypes/:key', () => {
    const appPath = path.resolve(
      __dirname,
      '../../../../app/src/App.tsx',
    );
    const content = fs.readFileSync(appPath, 'utf-8');

    // Get positions of routes
    const collectionsRoutePos = content.indexOf('path="/issuetypes/collections"');
    const keyRoutePos = content.indexOf('path="/issuetypes/:key"');

    // /issuetypes/collections should come before /issuetypes/:key
    expect(collectionsRoutePos).toBeLessThan(keyRoutePos);
  });
});

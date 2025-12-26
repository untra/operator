//! Backstage scaffold generator.
//!
//! Generates a minimal, Bun-compatible Backstage deployment including:
//! - package.json with workspace configuration
//! - app-config.yaml with guest auth and file:// catalog locations
//! - Docker files for containerized deployment
//! - Minimal app and backend packages

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::branding::{BrandingAssets, BrandingDefaults};
use super::taxonomy::Taxonomy;
use crate::config::Config;

/// Result of scaffold generation.
#[derive(Debug, Clone)]
pub struct ScaffoldResult {
    /// Files that were created.
    pub created: Vec<PathBuf>,
    /// Files that were skipped (already exist).
    pub skipped: Vec<PathBuf>,
    /// Errors encountered (path, error message).
    pub errors: Vec<(PathBuf, String)>,
    /// Output directory for the scaffold.
    pub output_dir: PathBuf,
}

impl ScaffoldResult {
    /// Create a new empty result.
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            created: Vec::new(),
            skipped: Vec::new(),
            errors: Vec::new(),
            output_dir,
        }
    }

    /// Check if scaffold completed without errors.
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    /// Generate a summary string.
    pub fn summary(&self) -> String {
        format!(
            "{} created, {} skipped, {} errors",
            self.created.len(),
            self.skipped.len(),
            self.errors.len()
        )
    }
}

/// Configuration for scaffold generation.
#[derive(Debug, Clone)]
pub struct ScaffoldOptions {
    /// Force overwrite existing files.
    pub force: bool,
    /// Custom branding name (e.g., "My Company Portal").
    pub branding_name: Option<String>,
    /// Generate Docker files.
    pub include_docker: bool,
    /// Port for Backstage server.
    pub port: u16,
    /// Projects directory to scan for catalog locations.
    pub projects_dir: PathBuf,
}

impl Default for ScaffoldOptions {
    fn default() -> Self {
        Self {
            force: false,
            branding_name: None,
            include_docker: true,
            port: 7007,
            projects_dir: PathBuf::from("."),
        }
    }
}

impl ScaffoldOptions {
    /// Create options from operator config.
    pub fn from_config(config: &Config) -> Self {
        Self {
            force: false,
            branding_name: None,
            include_docker: true,
            port: config.backstage.port,
            projects_dir: config.projects_path(),
        }
    }

    /// Get branding configuration.
    pub fn branding(&self) -> BrandingDefaults {
        match &self.branding_name {
            Some(name) => BrandingDefaults::with_name(name),
            None => BrandingDefaults::default(),
        }
    }
}

/// A single file to be scaffolded.
pub trait ScaffoldFile {
    /// Relative path within the backstage output directory.
    fn path(&self) -> &'static str;

    /// Generate the file content.
    fn generate(&self, options: &ScaffoldOptions) -> Result<String>;

    /// Whether this file should be created (allows conditional generation).
    fn should_create(&self, _options: &ScaffoldOptions) -> bool {
        true
    }
}

// =============================================================================
// FILE GENERATORS
// =============================================================================

/// Generates package.json for Bun/Node workspace.
pub struct PackageJsonGenerator;

impl ScaffoldFile for PackageJsonGenerator {
    fn path(&self) -> &'static str {
        "package.json"
    }

    fn generate(&self, options: &ScaffoldOptions) -> Result<String> {
        let name = options
            .branding_name
            .as_ref()
            .map(|n| n.to_lowercase().replace(' ', "-"))
            .unwrap_or_else(|| "backstage-local".to_string());

        Ok(format!(
            r#"{{
  "name": "{name}",
  "version": "1.0.0",
  "private": true,
  "engines": {{
    "node": ">=18"
  }},
  "scripts": {{
    "dev": "concurrently \"yarn start\" \"yarn start-backend\"",
    "start": "yarn workspace app start",
    "start-backend": "yarn workspace backend start",
    "build": "backstage-cli repo build --all",
    "build:backend": "yarn workspace backend build"
  }},
  "workspaces": {{
    "packages": [
      "packages/*",
      "packages/plugins/*"
    ]
  }},
  "devDependencies": {{
    "@backstage/cli": "^0.27.0",
    "concurrently": "^8.0.0"
  }},
  "resolutions": {{
    "@types/react": "^18"
  }}
}}"#,
            name = name
        ))
    }
}

/// Generates app-config.yaml with guest auth and taxonomy kinds.
pub struct AppConfigGenerator;

impl ScaffoldFile for AppConfigGenerator {
    fn path(&self) -> &'static str {
        "app-config.yaml"
    }

    fn generate(&self, options: &ScaffoldOptions) -> Result<String> {
        let taxonomy = Taxonomy::load();
        let branding = options.branding();

        // Generate allowed types from taxonomy (all 24 kinds)
        let kind_types: String = taxonomy
            .kinds
            .iter()
            .map(|k| format!("        - {}", k.key))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(format!(
            r#"app:
  title: {title}
  baseUrl: http://localhost:{port}

organization:
  name: {title}

backend:
  baseUrl: http://localhost:{port}
  listen:
    port: {port}
  database:
    client: better-sqlite3
    connection: ':memory:'

auth:
  providers:
    guest:
      dangerouslyAllowOutsideDevelopment: true

# Proxy configuration for Operator REST API
proxy:
  '/operator':
    target: 'http://localhost:7008'
    changeOrigin: true

catalog:
  rules:
    - allow:
        - Component
        - API
        - Resource
        - System
        - Domain
        - Location
        - Template
        - Group
        - User
  # Extended component types from operator taxonomy
  # See: https://backstage.io/docs/features/software-catalog/extending-the-model
  processors:
    catalogModuleKinds:
      allowedTypes:
{kind_types}
  locations:
    # Scan workspace for catalog-info.yaml files
    - type: file
      target: ../../**/catalog-info.yaml
      rules:
        - allow: [Component, API, Resource, System, Domain]
"#,
            title = branding.title,
            port = options.port,
            kind_types = kind_types,
        ))
    }
}

/// Generates app-config.production.yaml for Docker deployment.
pub struct AppConfigProductionGenerator;

impl ScaffoldFile for AppConfigProductionGenerator {
    fn path(&self) -> &'static str {
        "app-config.production.yaml"
    }

    fn generate(&self, options: &ScaffoldOptions) -> Result<String> {
        Ok(format!(
            r#"app:
  baseUrl: ${{BACKSTAGE_BASE_URL:-http://localhost:{port}}}

backend:
  baseUrl: ${{BACKSTAGE_BACKEND_URL:-http://localhost:{port}}}
  listen:
    port: {port}
  database:
    client: better-sqlite3
    connection: /app/data/backstage.sqlite

# Production should use proper auth, not guest
# Uncomment and configure for production use:
# auth:
#   providers:
#     github:
#       development:
#         clientId: ${{GITHUB_CLIENT_ID}}
#         clientSecret: ${{GITHUB_CLIENT_SECRET}}
"#,
            port = options.port
        ))
    }
}

/// Generates Dockerfile for containerized deployment.
pub struct DockerfileGenerator;

impl ScaffoldFile for DockerfileGenerator {
    fn path(&self) -> &'static str {
        "Dockerfile"
    }

    fn should_create(&self, options: &ScaffoldOptions) -> bool {
        options.include_docker
    }

    fn generate(&self, _options: &ScaffoldOptions) -> Result<String> {
        Ok(r#"# Stage 1: Build
FROM node:20-bookworm-slim AS builder

WORKDIR /app

# Install dependencies
COPY package.json yarn.lock ./
RUN corepack enable && yarn install --immutable

# Copy source and build
COPY . .
RUN yarn build

# Stage 2: Runtime
FROM node:20-bookworm-slim

WORKDIR /app

# Copy built artifacts
COPY --from=builder /app/packages/backend/dist /app/dist
COPY --from=builder /app/node_modules /app/node_modules
COPY app-config.yaml app-config.production.yaml ./

# Create data directory for SQLite
RUN mkdir -p /app/data && chown node:node /app/data

USER node

EXPOSE 7007

CMD ["node", "dist/index.cjs"]
"#
        .to_string())
    }
}

/// Generates docker-compose.yaml for local Docker orchestration.
pub struct DockerComposeGenerator;

impl ScaffoldFile for DockerComposeGenerator {
    fn path(&self) -> &'static str {
        "docker-compose.yaml"
    }

    fn should_create(&self, options: &ScaffoldOptions) -> bool {
        options.include_docker
    }

    fn generate(&self, options: &ScaffoldOptions) -> Result<String> {
        Ok(format!(
            r#"version: '3.8'

services:
  backstage:
    build: .
    ports:
      - "{port}:{port}"
    environment:
      - BACKSTAGE_BASE_URL=http://localhost:{port}
      - BACKSTAGE_BACKEND_URL=http://localhost:{port}
    volumes:
      - backstage-data:/app/data
      # Mount workspace for catalog scanning (read-only)
      - ../..:/workspace:ro
    restart: unless-stopped

volumes:
  backstage-data:
"#,
            port = options.port
        ))
    }
}

/// Generates README.md with usage instructions.
pub struct ReadmeGenerator;

impl ScaffoldFile for ReadmeGenerator {
    fn path(&self) -> &'static str {
        "README.md"
    }

    fn generate(&self, options: &ScaffoldOptions) -> Result<String> {
        let branding = options.branding();
        Ok(format!(
            r#"# {title}

A local Backstage deployment generated by operator.

## Quick Start

### Local Development (Bun/Yarn)

```bash
# Install dependencies
yarn install

# Start dev server
yarn dev
```

Open http://localhost:{port} in your browser.

### Docker

```bash
# Build and run
docker-compose up --build

# Or build only
docker build -t backstage-local .
docker run -p {port}:{port} backstage-local
```

## Configuration

- `app-config.yaml` - Main configuration (guest auth, local catalog)
- `app-config.production.yaml` - Production overrides (Docker)

## Catalog

The catalog scans `../../**/catalog-info.yaml` for components.
Create a `catalog-info.yaml` in your projects to register them.

## Branding

Customize the portal by editing:
- `branding/logo.svg` - Portal logo
- `app-config.yaml` - Title and colors

## Generated by

This scaffold was generated by [operator](https://github.com/untra/operator).
"#,
            title = branding.title,
            port = options.port
        ))
    }
}

/// Generates packages/app/package.json for frontend.
pub struct AppPackageGenerator;

impl ScaffoldFile for AppPackageGenerator {
    fn path(&self) -> &'static str {
        "packages/app/package.json"
    }

    fn generate(&self, _options: &ScaffoldOptions) -> Result<String> {
        Ok(r#"{
  "name": "app",
  "version": "0.0.0",
  "private": true,
  "bundled": true,
  "scripts": {
    "start": "backstage-cli package start"
  },
  "dependencies": {
    "@backstage/app-defaults": "^1.5.0",
    "@backstage/core-app-api": "^1.14.0",
    "@backstage/core-components": "^0.14.0",
    "@backstage/core-plugin-api": "^1.9.0",
    "@backstage/plugin-catalog": "^1.21.0",
    "@backstage/plugin-catalog-graph": "^0.4.0",
    "@backstage/plugin-catalog-import": "^0.12.0",
    "@backstage/plugin-catalog-react": "^1.12.0",
    "@backstage/theme": "^0.5.0",
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "react-router-dom": "^6.0.0"
  },
  "devDependencies": {
    "@backstage/cli": "^0.27.0"
  }
}
"#
        .to_string())
    }
}

/// Generates packages/backend/package.json for backend.
pub struct BackendPackageGenerator;

impl ScaffoldFile for BackendPackageGenerator {
    fn path(&self) -> &'static str {
        "packages/backend/package.json"
    }

    fn generate(&self, _options: &ScaffoldOptions) -> Result<String> {
        Ok(r#"{
  "name": "backend",
  "version": "0.0.0",
  "private": true,
  "main": "dist/index.cjs.js",
  "scripts": {
    "start": "backstage-cli package start",
    "build": "backstage-cli package build"
  },
  "dependencies": {
    "@backstage/backend-defaults": "^0.4.0",
    "@backstage/plugin-app-backend": "^0.3.0",
    "@backstage/plugin-auth-backend": "^0.22.0",
    "@backstage/plugin-auth-backend-module-guest-provider": "^0.1.0",
    "@backstage/plugin-catalog-backend": "^1.24.0",
    "@backstage/plugin-catalog-backend-module-scaffolder-entity-model": "^0.1.0",
    "@backstage/plugin-proxy-backend": "^0.5.0",
    "better-sqlite3": "^9.0.0"
  },
  "devDependencies": {
    "@backstage/cli": "^0.27.0"
  }
}
"#
        .to_string())
    }
}

/// Generates branding/logo.svg with default logo.
pub struct LogoGenerator;

impl ScaffoldFile for LogoGenerator {
    fn path(&self) -> &'static str {
        "branding/logo.svg"
    }

    fn generate(&self, _options: &ScaffoldOptions) -> Result<String> {
        Ok(BrandingAssets::default_logo_svg().to_string())
    }
}

/// Generates packages/app/src/App.tsx - React frontend entry component.
pub struct AppTsxGenerator;

impl ScaffoldFile for AppTsxGenerator {
    fn path(&self) -> &'static str {
        "packages/app/src/App.tsx"
    }

    fn generate(&self, _options: &ScaffoldOptions) -> Result<String> {
        Ok(r#"/**
 * Operator Backstage Frontend
 *
 * Catalog browser with issue types management.
 */

import React from 'react';
import { Route } from 'react-router-dom';
import { createApp, FlatRoutes } from '@backstage/app-defaults';
import { catalogPlugin } from '@backstage/plugin-catalog';
import {
  issueTypesPlugin,
  IssueTypesPage,
  IssueTypeDetailPage,
  IssueTypeFormPage,
  CollectionsPage,
} from '@operator/plugin-issuetypes';

const app = createApp({
  plugins: [catalogPlugin, issueTypesPlugin],
});

const AppProvider = app.getProvider();
const AppRouter = app.getRouter();

const routes = (
  <FlatRoutes>
    <Route path="/issuetypes" element={<IssueTypesPage />}>
      <Route path="new" element={<IssueTypeFormPage />} />
      <Route path="collections" element={<CollectionsPage />} />
      <Route path=":key" element={<IssueTypeDetailPage />} />
      <Route path=":key/edit" element={<IssueTypeFormPage />} />
    </Route>
  </FlatRoutes>
);

export default function App() {
  return (
    <AppProvider>
      <AppRouter>
        {routes}
      </AppRouter>
    </AppProvider>
  );
}
"#
        .to_string())
    }
}

/// Generates packages/app/src/index.tsx - React DOM entry point.
pub struct AppIndexTsxGenerator;

impl ScaffoldFile for AppIndexTsxGenerator {
    fn path(&self) -> &'static str {
        "packages/app/src/index.tsx"
    }

    fn generate(&self, _options: &ScaffoldOptions) -> Result<String> {
        Ok(r#"/**
 * Operator Backstage Frontend Entry Point
 */

import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';

const root = ReactDOM.createRoot(
  document.getElementById('root') as HTMLElement
);

root.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
"#
        .to_string())
    }
}

/// Generates packages/backend/src/index.ts - Bun backend entry point.
pub struct BackendIndexTsGenerator;

impl ScaffoldFile for BackendIndexTsGenerator {
    fn path(&self) -> &'static str {
        "packages/backend/src/index.ts"
    }

    fn generate(&self, _options: &ScaffoldOptions) -> Result<String> {
        Ok(r#"/**
 * Operator Backstage Backend
 *
 * Minimal Bun-based backend for local catalog browsing.
 * Includes proxy for Operator API integration.
 */

import { createBackend } from '@backstage/backend-defaults';

const backend = createBackend();

// Core plugins
backend.add(import('@backstage/plugin-app-backend'));
backend.add(import('@backstage/plugin-catalog-backend'));

// Proxy for Operator REST API
backend.add(import('@backstage/plugin-proxy-backend'));

// Start the backend
backend.start();
"#
        .to_string())
    }
}

/// Generates tsconfig.json for TypeScript configuration.
pub struct TsConfigGenerator;

impl ScaffoldFile for TsConfigGenerator {
    fn path(&self) -> &'static str {
        "tsconfig.json"
    }

    fn generate(&self, _options: &ScaffoldOptions) -> Result<String> {
        Ok(r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "lib": ["ES2022", "DOM"],
    "jsx": "react-jsx",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true,
    "outDir": "./dist",
    "rootDir": "./packages",
    "baseUrl": ".",
    "paths": {
      "backend": ["packages/backend/src"],
      "app": ["packages/app/src"]
    }
  },
  "include": ["packages/*/src/**/*"],
  "exclude": ["node_modules", "dist", "**/dist"]
}
"#
        .to_string())
    }
}

/// Generates bunfig.toml for Bun configuration.
pub struct BunfigGenerator;

impl ScaffoldFile for BunfigGenerator {
    fn path(&self) -> &'static str {
        "bunfig.toml"
    }

    fn generate(&self, _options: &ScaffoldOptions) -> Result<String> {
        Ok(r#"# Bun configuration for Operator Backstage
# https://bun.sh/docs/runtime/bunfig

[install]
# Use Bun's native lockfile format
lockfile = "bun.lockb"

# NPM registry
registry = "https://registry.npmjs.org/"

# Save dependencies to package.json
save = true
"#
        .to_string())
    }
}

// =============================================================================
// SCAFFOLD ORCHESTRATOR
// =============================================================================

/// Copy a directory recursively.
fn copy_dir_recursive(src: &Path, dst: &Path, force: bool) -> Result<Vec<PathBuf>> {
    let mut copied = Vec::new();

    if !src.exists() {
        return Ok(copied);
    }

    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            let sub_copied = copy_dir_recursive(&src_path, &dst_path, force)?;
            copied.extend(sub_copied);
        } else {
            // Skip if exists and not forcing
            if dst_path.exists() && !force {
                continue;
            }
            fs::copy(&src_path, &dst_path)?;
            copied.push(dst_path);
        }
    }

    Ok(copied)
}

/// The main scaffold generator.
pub struct BackstageScaffold {
    output_dir: PathBuf,
    options: ScaffoldOptions,
    /// Path to the operator source plugins directory (for copying)
    plugins_source: Option<PathBuf>,
}

impl BackstageScaffold {
    /// Create a new scaffold generator.
    pub fn new(output_dir: PathBuf, options: ScaffoldOptions) -> Self {
        Self {
            output_dir,
            options,
            plugins_source: None,
        }
    }

    /// Create a new scaffold generator with plugin source path.
    pub fn with_plugins_source(
        output_dir: PathBuf,
        options: ScaffoldOptions,
        plugins_source: PathBuf,
    ) -> Self {
        Self {
            output_dir,
            options,
            plugins_source: Some(plugins_source),
        }
    }

    /// Check if scaffold already exists.
    pub fn exists(output_dir: &Path) -> bool {
        output_dir.join("package.json").exists()
    }

    /// Get all file generators.
    fn generators(&self) -> Vec<Box<dyn ScaffoldFile>> {
        vec![
            Box::new(PackageJsonGenerator),
            Box::new(AppConfigGenerator),
            Box::new(AppConfigProductionGenerator),
            Box::new(DockerfileGenerator),
            Box::new(DockerComposeGenerator),
            Box::new(ReadmeGenerator),
            Box::new(AppPackageGenerator),
            Box::new(BackendPackageGenerator),
            Box::new(LogoGenerator),
            // TypeScript source files
            Box::new(AppTsxGenerator),
            Box::new(AppIndexTsxGenerator),
            Box::new(BackendIndexTsGenerator),
            Box::new(TsConfigGenerator),
            Box::new(BunfigGenerator),
        ]
    }

    /// Copy plugins from source directory to scaffold output.
    fn copy_plugins(&self, result: &mut ScaffoldResult) {
        if let Some(plugins_source) = &self.plugins_source {
            let plugins_dest = self.output_dir.join("packages/plugins");

            match copy_dir_recursive(plugins_source, &plugins_dest, self.options.force) {
                Ok(copied) => {
                    result.created.extend(copied);
                }
                Err(e) => {
                    result
                        .errors
                        .push((plugins_dest, format!("Failed to copy plugins: {}", e)));
                }
            }
        }
    }

    /// Generate all scaffold files.
    pub fn generate(&self) -> Result<ScaffoldResult> {
        let mut result = ScaffoldResult::new(self.output_dir.clone());

        // Create output directory
        fs::create_dir_all(&self.output_dir)
            .with_context(|| format!("Failed to create directory: {:?}", self.output_dir))?;

        for generator in self.generators() {
            // Check if this file should be created
            if !generator.should_create(&self.options) {
                continue;
            }

            let rel_path = generator.path();
            let full_path = self.output_dir.join(rel_path);

            // Check if file exists and skip if not forcing
            if full_path.exists() && !self.options.force {
                result.skipped.push(full_path);
                continue;
            }

            // Create parent directories
            if let Some(parent) = full_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    result.errors.push((
                        full_path.clone(),
                        format!("Failed to create directory: {}", e),
                    ));
                    continue;
                }
            }

            // Generate and write content
            match generator.generate(&self.options) {
                Ok(content) => {
                    if let Err(e) = fs::write(&full_path, content) {
                        result
                            .errors
                            .push((full_path, format!("Failed to write: {}", e)));
                    } else {
                        result.created.push(full_path);
                    }
                }
                Err(e) => {
                    result
                        .errors
                        .push((full_path, format!("Failed to generate: {}", e)));
                }
            }
        }

        // Copy plugins directory if source is provided
        self.copy_plugins(&mut result);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_scaffold_result_summary() {
        let mut result = ScaffoldResult::new(PathBuf::from("/test"));
        result.created.push(PathBuf::from("package.json"));
        result.skipped.push(PathBuf::from("existing.yaml"));
        assert!(result.summary().contains("1 created"));
        assert!(result.summary().contains("1 skipped"));
        assert!(result.is_success());
    }

    #[test]
    fn test_scaffold_result_with_errors() {
        let mut result = ScaffoldResult::new(PathBuf::from("/test"));
        result
            .errors
            .push((PathBuf::from("bad.txt"), "error".to_string()));
        assert!(!result.is_success());
    }

    #[test]
    fn test_scaffold_options_default() {
        let options = ScaffoldOptions::default();
        assert!(!options.force);
        assert!(options.include_docker);
        assert_eq!(options.port, 7007);
    }

    #[test]
    fn test_scaffold_options_branding() {
        let options = ScaffoldOptions {
            branding_name: Some("My Portal".to_string()),
            ..Default::default()
        };
        let branding = options.branding();
        assert_eq!(branding.title, "My Portal");
    }

    #[test]
    fn test_package_json_generator() {
        let gen = PackageJsonGenerator;
        let content = gen.generate(&ScaffoldOptions::default()).unwrap();
        assert!(content.contains("\"name\": \"backstage-local\""));
        assert!(content.contains("backstage-cli"));
        assert!(content.contains("workspaces"));
    }

    #[test]
    fn test_package_json_with_custom_name() {
        let gen = PackageJsonGenerator;
        let options = ScaffoldOptions {
            branding_name: Some("My Company Portal".to_string()),
            ..Default::default()
        };
        let content = gen.generate(&options).unwrap();
        assert!(content.contains("\"name\": \"my-company-portal\""));
    }

    #[test]
    fn test_app_config_includes_all_kinds() {
        let gen = AppConfigGenerator;
        let content = gen.generate(&ScaffoldOptions::default()).unwrap();

        // Verify all kinds are present (flexible - works with any number)
        let taxonomy = Taxonomy::load();
        assert!(!taxonomy.kinds.is_empty(), "Should have kinds");
        for kind in &taxonomy.kinds {
            assert!(content.contains(&kind.key), "Missing kind: {}", kind.key);
        }
    }

    #[test]
    fn test_app_config_has_guest_auth() {
        let gen = AppConfigGenerator;
        let content = gen.generate(&ScaffoldOptions::default()).unwrap();
        assert!(content.contains("guest:"));
        assert!(content.contains("dangerouslyAllowOutsideDevelopment: true"));
    }

    #[test]
    fn test_dockerfile_conditional_creation() {
        let gen = DockerfileGenerator;

        let with_docker = ScaffoldOptions {
            include_docker: true,
            ..Default::default()
        };
        assert!(gen.should_create(&with_docker));

        let without_docker = ScaffoldOptions {
            include_docker: false,
            ..Default::default()
        };
        assert!(!gen.should_create(&without_docker));
    }

    #[test]
    fn test_dockerfile_content() {
        let gen = DockerfileGenerator;
        let content = gen.generate(&ScaffoldOptions::default()).unwrap();
        assert!(content.contains("FROM node:20-bookworm-slim"));
        assert!(content.contains("EXPOSE 7007"));
    }

    #[test]
    fn test_docker_compose_uses_port() {
        let gen = DockerComposeGenerator;
        let options = ScaffoldOptions {
            port: 8080,
            ..Default::default()
        };
        let content = gen.generate(&options).unwrap();
        assert!(content.contains("8080:8080"));
    }

    #[test]
    fn test_readme_generator() {
        let gen = ReadmeGenerator;
        let content = gen.generate(&ScaffoldOptions::default()).unwrap();
        assert!(content.contains("Developer Portal"));
        assert!(content.contains("yarn dev"));
        assert!(content.contains("docker-compose"));
    }

    #[test]
    fn test_logo_generator() {
        let gen = LogoGenerator;
        let content = gen.generate(&ScaffoldOptions::default()).unwrap();
        assert!(content.contains("<svg"));
        assert!(content.contains("</svg>"));
    }

    #[test]
    fn test_app_tsx_generator() {
        let gen = AppTsxGenerator;
        let content = gen.generate(&ScaffoldOptions::default()).unwrap();
        assert!(content.contains("import React from 'react'"));
        assert!(content.contains("@backstage/app-defaults"));
        assert!(content.contains("catalogPlugin"));
        assert!(content.contains("export default function App()"));
    }

    #[test]
    fn test_app_index_tsx_generator() {
        let gen = AppIndexTsxGenerator;
        let content = gen.generate(&ScaffoldOptions::default()).unwrap();
        assert!(content.contains("ReactDOM.createRoot"));
        assert!(content.contains("import App from './App'"));
        assert!(content.contains("<React.StrictMode>"));
    }

    #[test]
    fn test_backend_index_ts_generator() {
        let gen = BackendIndexTsGenerator;
        let content = gen.generate(&ScaffoldOptions::default()).unwrap();
        assert!(content.contains("@backstage/backend-defaults"));
        assert!(content.contains("createBackend"));
        assert!(content.contains("plugin-app-backend"));
        assert!(content.contains("plugin-catalog-backend"));
        assert!(content.contains("backend.start()"));
    }

    #[test]
    fn test_tsconfig_generator() {
        let gen = TsConfigGenerator;
        let content = gen.generate(&ScaffoldOptions::default()).unwrap();
        assert!(content.contains("\"target\": \"ES2022\""));
        assert!(content.contains("\"jsx\": \"react-jsx\""));
        assert!(content.contains("\"moduleResolution\": \"bundler\""));
    }

    #[test]
    fn test_bunfig_generator() {
        let gen = BunfigGenerator;
        let content = gen.generate(&ScaffoldOptions::default()).unwrap();
        assert!(content.contains("[install]"));
        assert!(content.contains("lockfile = \"bun.lockb\""));
        assert!(content.contains("registry.npmjs.org"));
    }

    #[test]
    fn test_scaffold_exists() {
        let temp_dir = TempDir::new().unwrap();
        assert!(!BackstageScaffold::exists(temp_dir.path()));

        // Create package.json
        fs::write(temp_dir.path().join("package.json"), "{}").unwrap();
        assert!(BackstageScaffold::exists(temp_dir.path()));
    }

    #[test]
    fn test_scaffold_generate() {
        let temp_dir = TempDir::new().unwrap();
        let scaffold =
            BackstageScaffold::new(temp_dir.path().to_path_buf(), ScaffoldOptions::default());

        let result = scaffold.generate().unwrap();
        assert!(result.is_success());
        assert!(!result.created.is_empty());
        assert!(result.skipped.is_empty());

        // Verify key files exist
        assert!(temp_dir.path().join("package.json").exists());
        assert!(temp_dir.path().join("app-config.yaml").exists());
        assert!(temp_dir.path().join("Dockerfile").exists());
        assert!(temp_dir.path().join("docker-compose.yaml").exists());
        assert!(temp_dir.path().join("README.md").exists());
        assert!(temp_dir.path().join("packages/app/package.json").exists());
        assert!(temp_dir
            .path()
            .join("packages/backend/package.json")
            .exists());
        assert!(temp_dir.path().join("branding/logo.svg").exists());

        // Verify TypeScript source files exist
        assert!(temp_dir.path().join("packages/app/src/App.tsx").exists());
        assert!(temp_dir.path().join("packages/app/src/index.tsx").exists());
        assert!(temp_dir
            .path()
            .join("packages/backend/src/index.ts")
            .exists());
        assert!(temp_dir.path().join("tsconfig.json").exists());
        assert!(temp_dir.path().join("bunfig.toml").exists());
    }

    #[test]
    fn test_scaffold_idempotency() {
        let temp_dir = TempDir::new().unwrap();
        let scaffold =
            BackstageScaffold::new(temp_dir.path().to_path_buf(), ScaffoldOptions::default());

        // First run - creates files
        let result1 = scaffold.generate().unwrap();
        assert!(!result1.created.is_empty());
        assert!(result1.skipped.is_empty());

        // Second run - skips existing
        let result2 = scaffold.generate().unwrap();
        assert!(result2.created.is_empty());
        assert!(!result2.skipped.is_empty());
    }

    #[test]
    fn test_scaffold_force_overwrites() {
        let temp_dir = TempDir::new().unwrap();

        // First run without force
        let scaffold1 =
            BackstageScaffold::new(temp_dir.path().to_path_buf(), ScaffoldOptions::default());
        scaffold1.generate().unwrap();

        // Second run with force
        let scaffold2 = BackstageScaffold::new(
            temp_dir.path().to_path_buf(),
            ScaffoldOptions {
                force: true,
                ..Default::default()
            },
        );
        let result = scaffold2.generate().unwrap();
        assert!(!result.created.is_empty());
        assert!(result.skipped.is_empty());
    }

    #[test]
    fn test_scaffold_without_docker() {
        let temp_dir = TempDir::new().unwrap();
        let scaffold = BackstageScaffold::new(
            temp_dir.path().to_path_buf(),
            ScaffoldOptions {
                include_docker: false,
                ..Default::default()
            },
        );

        let result = scaffold.generate().unwrap();
        assert!(result.is_success());

        // Docker files should not exist
        assert!(!temp_dir.path().join("Dockerfile").exists());
        assert!(!temp_dir.path().join("docker-compose.yaml").exists());

        // Other files should exist
        assert!(temp_dir.path().join("package.json").exists());
        assert!(temp_dir.path().join("app-config.yaml").exists());
    }
}

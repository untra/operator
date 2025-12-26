---
title: "Project Taxonomy"
layout: doc
---

<!-- AUTO-GENERATED FROM src/backstage/taxonomy.toml - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->

# Project Taxonomy

This document defines the **25 project Kinds** organized into **5 tiers**.

Each Kind represents a category of project that can be cataloged in Backstage. The taxonomy is used by the `ASSESS` issue type to classify projects and generate `catalog-info.yaml` files.

## Version

- **Version**: `1.0.0`
- **Description**: Operator project taxonomy for Backstage catalog

## Quick Reference

All 24 Kinds at a glance:

| ID | Key | Name | Tier | Backstage Type |
| --- | --- | --- | --- | --- |
| 1 | `infrastructure` | Infrastructure (IaC) | foundation | `resource` |
| 2 | `identity-access` | Identity & Access (IAM) | foundation | `resource` |
| 3 | `config-policy` | Config & Policy | foundation | `resource` |
| 4 | `monorepo-meta` | Monorepo / Meta | foundation | `system` |
| 5 | `design-system` | Design Systems | standards | `library` |
| 6 | `software-library` | Software Libraries | standards | `library` |
| 7 | `proto-sdk` | Proto / SDK | standards | `api` |
| 8 | `blueprint` | Blueprints | standards | `template` |
| 9 | `security-tooling` | Security Tooling | standards | `tool` |
| 10 | `compliance-audit` | Compliance / Audit | standards | `documentation` |
| 11 | `ml-model` | ML / Models | engines | `service` |
| 12 | `data-etl` | Data / ETL | engines | `service` |
| 13 | `microservice` | Microservices | engines | `service` |
| 14 | `api-gateway` | APIs / Gateways | engines | `api` |
| 15 | `ui-frontend` | UIs / Frontends | engines | `website` |
| 16 | `internal-tool` | Internal Tooling | engines | `service` |
| 17 | `build-tool` | Build Tools | ecosystem | `tool` |
| 18 | `e2e-test` | E2E Test Suites | ecosystem | `tool` |
| 19 | `docs-site` | Docs Sites | ecosystem | `website` |
| 20 | `playbook` | Internal Playbooks | ecosystem | `documentation` |
| 21 | `cli-devtool` | CLIs / Developer Tools | ecosystem | `tool` |
| 22 | `reference-example` | Reference / Example | noncurrent | `documentation` |
| 23 | `experiment-sandbox` | Experiment / Sandbox | noncurrent | `service` |
| 24 | `archival-fork` | Archival / Forks | noncurrent | `library` |
| 25 | `test-data-fixtures` | Test Data / Fixtures | noncurrent | `resource` |

## Tier: Foundation (Kinds 1-4)

Infrastructure and platform foundations

| ID | Key | Name | Stakeholder | Output |
| --- | --- | --- | --- | --- |
| 1 | `infrastructure` | Infrastructure (IaC) | Platform/DevOps | Cloud Environment |
| 2 | `identity-access` | Identity & Access (IAM) | SDET/SecOps | Permissions/Tokens |
| 3 | `config-policy` | Config & Policy | Platform/DevOps | Runtime Behavior |
| 4 | `monorepo-meta` | Monorepo / Meta | Architect/Lead | Project Standards |

### 1 - Infrastructure (IaC)

Cloud resources and network (Terraform/CDK)

- **Key**: `infrastructure`
- **Stakeholder**: Platform/DevOps
- **Primary Output**: Cloud Environment
- **Backstage Type**: `resource`

**Detection** File Patterns:
- `*.tf`
- `*.tfvars`
- `cdk.json`
- `cdk.context.json`
- `pulumi.yaml`
- `Pulumi.yaml`
- `cloudformation.yaml`
- `cloudformation.yml`
- `serverless.yml`
- `terraform.tfstate`

### 2 - Identity & Access (IAM)

Service accounts, secrets, and RBAC policies

- **Key**: `identity-access`
- **Stakeholder**: SDET/SecOps
- **Primary Output**: Permissions/Tokens
- **Backstage Type**: `resource`

**Detection** File Patterns:
- `iam-*.yaml`
- `iam-*.yml`
- `rbac*.yaml`
- `rbac*.yml`
- `policies/*.json`
- `policies/*.yaml`
- `service-account*.yaml`
- `.vault/*`
- `vault-*.hcl`

### 3 - Config & Policy

Global feature flags and environment manifests

- **Key**: `config-policy`
- **Stakeholder**: Platform/DevOps
- **Primary Output**: Runtime Behavior
- **Backstage Type**: `resource`

**Detection** File Patterns:
- `config/*.yaml`
- `config/*.yml`
- `config/*.toml`
- `environments/*.yaml`
- `environments/*.yml`
- `.env.example`
- `feature-flags.yaml`
- `feature-flags.json`
- `launchdarkly*.yaml`

### 4 - Monorepo / Meta

Orchestration for projects and root standards

- **Key**: `monorepo-meta`
- **Stakeholder**: Architect/Lead
- **Primary Output**: Project Standards
- **Backstage Type**: `system`

**Detection** File Patterns:
- `nx.json`
- `turbo.json`
- `lerna.json`
- `pnpm-workspace.yaml`
- `rush.json`
- `Cargo.toml`
- `.github/CODEOWNERS`
- `CLAUDE.md`
- `CONTRIBUTING.md`

## Tier: Standards (Kinds 5-10)

Shared components and specifications

| ID | Key | Name | Stakeholder | Output |
| --- | --- | --- | --- | --- |
| 5 | `design-system` | Design Systems | Product/UX | Component Libraries |
| 6 | `software-library` | Software Libraries | Engineering | Versioned Packages |
| 7 | `proto-sdk` | Proto / SDK | Engineering | Contract Libraries |
| 8 | `blueprint` | Blueprints | Architect/Lead | Project Templates |
| 9 | `security-tooling` | Security Tooling | SDET/SecOps | Security Reports |
| 10 | `compliance-audit` | Compliance / Audit | SDET/SecOps | Compliance Proofs |

### 5 - Design Systems

Reusable UI components and brand tokens

- **Key**: `design-system`
- **Stakeholder**: Product/UX
- **Primary Output**: Component Libraries
- **Backstage Type**: `library`

**Detection** File Patterns:
- `tokens/*.json`
- `tokens/*.yaml`
- `design-tokens/*`
- `.storybook/*`
- `stories/*.stories.*`
- `components/*.stories.*`
- `figma-tokens.json`
- `style-dictionary.config.*`

### 6 - Software Libraries

Reusable internal logic packages (Shared Utils)

- **Key**: `software-library`
- **Stakeholder**: Engineering
- **Primary Output**: Versioned Packages
- **Backstage Type**: `library`

**Detection** File Patterns:
- `lib/*`
- `packages/*/package.json`
- `crates/*/Cargo.toml`
- `src/lib.rs`
- `index.ts`
- `index.js`
- `setup.py`
- `pyproject.toml`

### 7 - Proto / SDK

API contracts and generated client libraries

- **Key**: `proto-sdk`
- **Stakeholder**: Engineering
- **Primary Output**: Contract Libraries
- **Backstage Type**: `api`

**Detection** File Patterns:
- `*.proto`
- `proto/*`
- `protos/*`
- `openapi.yaml`
- `openapi.json`
- `swagger.yaml`
- `swagger.json`
- `graphql.schema`
- `schema.graphql`
- `*.thrift`
- `buf.yaml`
- `buf.gen.yaml`

### 8 - Blueprints

Scaffolding templates for bootstrapping repos

- **Key**: `blueprint`
- **Stakeholder**: Architect/Lead
- **Primary Output**: Project Templates
- **Backstage Type**: `template`

**Detection** File Patterns:
- `template.yaml`
- `cookiecutter.json`
- `copier.yaml`
- `copier.yml`
- `yeoman-generator/*`
- `skeleton/*`
- `blueprint/*`
- `.scaffold/*`

### 9 - Security Tooling

Custom scanners, audit scripts, and honeytokens

- **Key**: `security-tooling`
- **Stakeholder**: SDET/SecOps
- **Primary Output**: Security Reports
- **Backstage Type**: `tool`

**Detection** File Patterns:
- `security/*`
- `audit/*`
- `scanners/*`
- `.snyk`
- `trivy.yaml`
- `semgrep.yaml`
- `semgrep.yml`
- `.gitleaks.toml`
- `bandit.yaml`
- `safety/*`

### 10 - Compliance / Audit

Evidence, snapshots, and regulatory reports

- **Key**: `compliance-audit`
- **Stakeholder**: SDET/SecOps
- **Primary Output**: Compliance Proofs
- **Backstage Type**: `documentation`

**Detection** File Patterns:
- `compliance/*`
- `audit-logs/*`
- `evidence/*`
- `soc2/*`
- `hipaa/*`
- `gdpr/*`
- `pci-dss/*`
- `attestations/*`
- `controls/*.yaml`

## Tier: Engines (Kinds 11-16)

Core business logic and services

| ID | Key | Name | Stakeholder | Output |
| --- | --- | --- | --- | --- |
| 11 | `ml-model` | ML / Models | Data/ML | Model Artifacts |
| 12 | `data-etl` | Data / ETL | Data/ML | Clean Datasets |
| 13 | `microservice` | Microservices | Engineering | Running Binaries |
| 14 | `api-gateway` | APIs / Gateways | Engineering | Network Endpoints |
| 15 | `ui-frontend` | UIs / Frontends | Engineering | Web/Mobile Assets |
| 16 | `internal-tool` | Internal Tooling | Engineering | Operational Apps |

### 11 - ML / Models

Training scripts and model weight artifacts

- **Key**: `ml-model`
- **Stakeholder**: Data/ML
- **Primary Output**: Model Artifacts
- **Backstage Type**: `service`

**Detection** File Patterns:
- `model/*`
- `models/*`
- `training/*`
- `*.h5`
- `*.pt`
- `*.pth`
- `*.onnx`
- `*.pkl`
- `mlflow.yaml`
- `MLproject`
- `dvc.yaml`
- `dvc.lock`

### 12 - Data / ETL

Data transformation logic and SQL models

- **Key**: `data-etl`
- **Stakeholder**: Data/ML
- **Primary Output**: Clean Datasets
- **Backstage Type**: `service`

**Detection** File Patterns:
- `dbt_project.yml`
- `models/*.sql`
- `transforms/*`
- `pipelines/*`
- `airflow/*`
- `dags/*`
- `prefect.yaml`
- `dagster.yaml`
- `fivetran/*`

### 13 - Microservices

Backend business logic and domain units

- **Key**: `microservice`
- **Stakeholder**: Engineering
- **Primary Output**: Running Binaries
- **Backstage Type**: `service`

**Detection** File Patterns:
- `src/main.rs`
- `main.go`
- `cmd/main.go`
- `src/main/java/*`
- `main.py`
- `app.py`
- `server.ts`
- `server.js`
- `Dockerfile`
- `docker-compose.yml`

### 14 - APIs / Gateways

Entry points that route and protect traffic

- **Key**: `api-gateway`
- **Stakeholder**: Engineering
- **Primary Output**: Network Endpoints
- **Backstage Type**: `api`

**Detection** File Patterns:
- `gateway/*`
- `api-gateway/*`
- `kong.yaml`
- `kong.yml`
- `nginx.conf`
- `envoy.yaml`
- `traefik.yaml`
- `traefik.yml`
- `routes/*`
- `api/*.yaml`

### 15 - UIs / Frontends

Web or mobile apps for end-user interaction

- **Key**: `ui-frontend`
- **Stakeholder**: Engineering
- **Primary Output**: Web/Mobile Assets
- **Backstage Type**: `website`

**Detection** File Patterns:
- `src/App.tsx`
- `src/App.jsx`
- `src/App.vue`
- `src/App.svelte`
- `pages/*`
- `app/*`
- `next.config.*`
- `nuxt.config.*`
- `vite.config.*`
- `angular.json`
- `expo/*`

### 16 - Internal Tooling

Private apps for internal business operations

- **Key**: `internal-tool`
- **Stakeholder**: Engineering
- **Primary Output**: Operational Apps
- **Backstage Type**: `service`

**Detection** File Patterns:
- `admin/*`
- `backoffice/*`
- `internal/*`
- `dashboard/*`
- `retool/*`
- `metabase/*`

## Tier: Ecosystem (Kinds 17-21)

Supporting tools and utilities

| ID | Key | Name | Stakeholder | Output |
| --- | --- | --- | --- | --- |
| 17 | `build-tool` | Build Tools | Platform/DevOps | Automated Pipelines |
| 18 | `e2e-test` | E2E Test Suites | SDET/SecOps | Quality Reports |
| 19 | `docs-site` | Docs Sites | Product/UX | Static Support Sites |
| 20 | `playbook` | Internal Playbooks | Platform/DevOps | Operational Guides |
| 21 | `cli-devtool` | CLIs / Developer Tools | Platform/DevOps | Developer UX Tools |

### 17 - Build Tools

CI/CD actions and custom build logic

- **Key**: `build-tool`
- **Stakeholder**: Platform/DevOps
- **Primary Output**: Automated Pipelines
- **Backstage Type**: `tool`

**Detection** File Patterns:
- `.github/workflows/*`
- `.github/actions/*`
- `.gitlab-ci.yml`
- `Jenkinsfile`
- `.circleci/*`
- `azure-pipelines.yml`
- `buildkite/*`
- `.drone.yml`
- `Makefile`
- `Taskfile.yml`

### 18 - E2E Test Suites

Integration tests and smoke test runners

- **Key**: `e2e-test`
- **Stakeholder**: SDET/SecOps
- **Primary Output**: Quality Reports
- **Backstage Type**: `tool`

**Detection** File Patterns:
- `e2e/*`
- `tests/e2e/*`
- `integration/*`
- `tests/integration/*`
- `playwright.config.*`
- `cypress.config.*`
- `cypress/*`
- `playwright/*`
- `k6/*`
- `locust/*`

### 19 - Docs Sites

Documentation, tutorials, and references

- **Key**: `docs-site`
- **Stakeholder**: Product/UX
- **Primary Output**: Static Support Sites
- **Backstage Type**: `website`

**Detection** File Patterns:
- `docs/*`
- `docusaurus.config.*`
- `mkdocs.yml`
- `mkdocs.yaml`
- `_config.yml`
- `hugo.toml`
- `hugo.yaml`
- `sphinx/*`
- `conf.py`
- `book.toml`

### 20 - Internal Playbooks

Incident response and on-call runbooks

- **Key**: `playbook`
- **Stakeholder**: Platform/DevOps
- **Primary Output**: Operational Guides
- **Backstage Type**: `documentation`

**Detection** File Patterns:
- `playbooks/*`
- `runbooks/*`
- `oncall/*`
- `incidents/*`
- `postmortems/*`
- `sops/*`
- `procedures/*`

### 21 - CLIs / Developer Tools

Productivity scripts and developer utilities

- **Key**: `cli-devtool`
- **Stakeholder**: Platform/DevOps
- **Primary Output**: Developer UX Tools
- **Backstage Type**: `tool`

**Detection** File Patterns:
- `cli/*`
- `bin/*`
- `scripts/*`
- `tools/*`
- `devtools/*`
- `*.sh`
- `Justfile`

## Tier: Noncurrent (Kinds 22-25)

Repos of little product or operational importance (test data, examples, archives, forks)

| ID | Key | Name | Stakeholder | Output |
| --- | --- | --- | --- | --- |
| 22 | `reference-example` | Reference / Example | Architect/Lead | Educational Code |
| 23 | `experiment-sandbox` | Experiment / Sandbox | Engineering | Discardable Code |
| 24 | `archival-fork` | Archival / Forks | SDET/SecOps | Historical/Vendor Code |
| 25 | `test-data-fixtures` | Test Data / Fixtures | SDET/SecOps | Test Data Assets |

### 22 - Reference / Example

Best-practice implementation examples

- **Key**: `reference-example`
- **Stakeholder**: Architect/Lead
- **Primary Output**: Educational Code
- **Backstage Type**: `documentation`

**Detection** File Patterns:
- `examples/*`
- `samples/*`
- `demo/*`
- `tutorials/*`
- `quickstart/*`
- `getting-started/*`

### 23 - Experiment / Sandbox

Proof-of-concepts and R&D "spikes"

- **Key**: `experiment-sandbox`
- **Stakeholder**: Engineering
- **Primary Output**: Discardable Code
- **Backstage Type**: `service`

**Detection** File Patterns:
- `experiments/*`
- `sandbox/*`
- `spikes/*`
- `poc/*`
- `proof-of-concept/*`
- `scratch/*`
- `playground/*`

### 24 - Archival / Forks

Legacy code and forks of 3rd party repos

- **Key**: `archival-fork`
- **Stakeholder**: SDET/SecOps
- **Primary Output**: Historical/Vendor Code
- **Backstage Type**: `library`

**Detection** File Patterns:
- `vendor/*`
- `third-party/*`
- `3rdparty/*`
- `external/*`
- `archive/*`
- `legacy/*`
- `deprecated/*`

### 25 - Test Data / Fixtures

Repositories containing test data, fixtures, seed data, and mock datasets

- **Key**: `test-data-fixtures`
- **Stakeholder**: SDET/SecOps
- **Primary Output**: Test Data Assets
- **Backstage Type**: `resource`

**Detection** File Patterns:
- `fixtures/*`
- `testdata/*`
- `test-data/*`
- `seeds/*`
- `seed-data/*`
- `mocks/*`
- `mock-data/*`
- `sample-data/*`
- `*.fixtures.json`
- `*.fixture.json`
- `*.seed.sql`
- `db/seeds/*`

## File Pattern Detection

The taxonomy uses file pattern matching to suggest project Kinds. When analyzing a project, patterns are matched against file paths, and the Kind with the most matches is suggested.

### Pattern Syntax

Patterns use glob syntax:

- `*` - Match any characters except `/`
- `**` - Match any characters including `/`
- `?` - Match any single character
- `[abc]` - Match any character in brackets

### All Patterns by Kind

**Infrastructure (IaC)** (`infrastructure`):
- `*.tf`
- `*.tfvars`
- `cdk.json`
- `cdk.context.json`
- `pulumi.yaml`
- `Pulumi.yaml`
- `cloudformation.yaml`
- `cloudformation.yml`
- `serverless.yml`
- `terraform.tfstate`

**Identity & Access (IAM)** (`identity-access`):
- `iam-*.yaml`
- `iam-*.yml`
- `rbac*.yaml`
- `rbac*.yml`
- `policies/*.json`
- `policies/*.yaml`
- `service-account*.yaml`
- `.vault/*`
- `vault-*.hcl`

**Config & Policy** (`config-policy`):
- `config/*.yaml`
- `config/*.yml`
- `config/*.toml`
- `environments/*.yaml`
- `environments/*.yml`
- `.env.example`
- `feature-flags.yaml`
- `feature-flags.json`
- `launchdarkly*.yaml`

**Monorepo / Meta** (`monorepo-meta`):
- `nx.json`
- `turbo.json`
- `lerna.json`
- `pnpm-workspace.yaml`
- `rush.json`
- `Cargo.toml`
- `.github/CODEOWNERS`
- `CLAUDE.md`
- `CONTRIBUTING.md`

**Design Systems** (`design-system`):
- `tokens/*.json`
- `tokens/*.yaml`
- `design-tokens/*`
- `.storybook/*`
- `stories/*.stories.*`
- `components/*.stories.*`
- `figma-tokens.json`
- `style-dictionary.config.*`

**Software Libraries** (`software-library`):
- `lib/*`
- `packages/*/package.json`
- `crates/*/Cargo.toml`
- `src/lib.rs`
- `index.ts`
- `index.js`
- `setup.py`
- `pyproject.toml`

**Proto / SDK** (`proto-sdk`):
- `*.proto`
- `proto/*`
- `protos/*`
- `openapi.yaml`
- `openapi.json`
- `swagger.yaml`
- `swagger.json`
- `graphql.schema`
- `schema.graphql`
- `*.thrift`
- `buf.yaml`
- `buf.gen.yaml`

**Blueprints** (`blueprint`):
- `template.yaml`
- `cookiecutter.json`
- `copier.yaml`
- `copier.yml`
- `yeoman-generator/*`
- `skeleton/*`
- `blueprint/*`
- `.scaffold/*`

**Security Tooling** (`security-tooling`):
- `security/*`
- `audit/*`
- `scanners/*`
- `.snyk`
- `trivy.yaml`
- `semgrep.yaml`
- `semgrep.yml`
- `.gitleaks.toml`
- `bandit.yaml`
- `safety/*`

**Compliance / Audit** (`compliance-audit`):
- `compliance/*`
- `audit-logs/*`
- `evidence/*`
- `soc2/*`
- `hipaa/*`
- `gdpr/*`
- `pci-dss/*`
- `attestations/*`
- `controls/*.yaml`

**ML / Models** (`ml-model`):
- `model/*`
- `models/*`
- `training/*`
- `*.h5`
- `*.pt`
- `*.pth`
- `*.onnx`
- `*.pkl`
- `mlflow.yaml`
- `MLproject`
- `dvc.yaml`
- `dvc.lock`

**Data / ETL** (`data-etl`):
- `dbt_project.yml`
- `models/*.sql`
- `transforms/*`
- `pipelines/*`
- `airflow/*`
- `dags/*`
- `prefect.yaml`
- `dagster.yaml`
- `fivetran/*`

**Microservices** (`microservice`):
- `src/main.rs`
- `main.go`
- `cmd/main.go`
- `src/main/java/*`
- `main.py`
- `app.py`
- `server.ts`
- `server.js`
- `Dockerfile`
- `docker-compose.yml`

**APIs / Gateways** (`api-gateway`):
- `gateway/*`
- `api-gateway/*`
- `kong.yaml`
- `kong.yml`
- `nginx.conf`
- `envoy.yaml`
- `traefik.yaml`
- `traefik.yml`
- `routes/*`
- `api/*.yaml`

**UIs / Frontends** (`ui-frontend`):
- `src/App.tsx`
- `src/App.jsx`
- `src/App.vue`
- `src/App.svelte`
- `pages/*`
- `app/*`
- `next.config.*`
- `nuxt.config.*`
- `vite.config.*`
- `angular.json`
- `expo/*`

**Internal Tooling** (`internal-tool`):
- `admin/*`
- `backoffice/*`
- `internal/*`
- `dashboard/*`
- `retool/*`
- `metabase/*`

**Build Tools** (`build-tool`):
- `.github/workflows/*`
- `.github/actions/*`
- `.gitlab-ci.yml`
- `Jenkinsfile`
- `.circleci/*`
- `azure-pipelines.yml`
- `buildkite/*`
- `.drone.yml`
- `Makefile`
- `Taskfile.yml`

**E2E Test Suites** (`e2e-test`):
- `e2e/*`
- `tests/e2e/*`
- `integration/*`
- `tests/integration/*`
- `playwright.config.*`
- `cypress.config.*`
- `cypress/*`
- `playwright/*`
- `k6/*`
- `locust/*`

**Docs Sites** (`docs-site`):
- `docs/*`
- `docusaurus.config.*`
- `mkdocs.yml`
- `mkdocs.yaml`
- `_config.yml`
- `hugo.toml`
- `hugo.yaml`
- `sphinx/*`
- `conf.py`
- `book.toml`

**Internal Playbooks** (`playbook`):
- `playbooks/*`
- `runbooks/*`
- `oncall/*`
- `incidents/*`
- `postmortems/*`
- `sops/*`
- `procedures/*`

**CLIs / Developer Tools** (`cli-devtool`):
- `cli/*`
- `bin/*`
- `scripts/*`
- `tools/*`
- `devtools/*`
- `*.sh`
- `Justfile`

**Reference / Example** (`reference-example`):
- `examples/*`
- `samples/*`
- `demo/*`
- `tutorials/*`
- `quickstart/*`
- `getting-started/*`

**Experiment / Sandbox** (`experiment-sandbox`):
- `experiments/*`
- `sandbox/*`
- `spikes/*`
- `poc/*`
- `proof-of-concept/*`
- `scratch/*`
- `playground/*`

**Archival / Forks** (`archival-fork`):
- `vendor/*`
- `third-party/*`
- `3rdparty/*`
- `external/*`
- `archive/*`
- `legacy/*`
- `deprecated/*`

**Test Data / Fixtures** (`test-data-fixtures`):
- `fixtures/*`
- `testdata/*`
- `test-data/*`
- `seeds/*`
- `seed-data/*`
- `mocks/*`
- `mock-data/*`
- `sample-data/*`
- `*.fixtures.json`
- `*.fixture.json`
- `*.seed.sql`
- `db/seeds/*`

## Backstage Type Mapping

Each Kind maps to a Backstage catalog type:

| Backstage Type | Kinds |
| --- | --- |
| `api` | `proto-sdk`, `api-gateway` |
| `documentation` | `compliance-audit`, `playbook`, `reference-example` |
| `library` | `design-system`, `software-library`, `archival-fork` |
| `resource` | `infrastructure`, `identity-access`, `config-policy`, `test-data-fixtures` |
| `service` | `ml-model`, `data-etl`, `microservice`, `internal-tool`, `experiment-sandbox` |
| `system` | `monorepo-meta` |
| `template` | `blueprint` |
| `tool` | `security-tooling`, `build-tool`, `e2e-test`, `cli-devtool` |
| `website` | `ui-frontend`, `docs-site` |


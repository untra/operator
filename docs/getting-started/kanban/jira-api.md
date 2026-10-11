---
title: "Jira API Reference"
layout: doc
---

<!-- AUTO-GENERATED FROM docs/schemas/jira-api.json - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->

Auto-generated documentation of Jira Cloud REST API response types used by Operator.

## Overview

Operator integrates with the following Jira Cloud REST API endpoints:

| Endpoint | Description |
|----------|-------------|
| `GET /rest/api/3/user/assignable/search` | List assignable users |
| `GET /rest/api/3/project/{key}/statuses` | List project statuses |
| `GET /rest/api/3/search` | Search issues with JQL |

## JiraSearchResponse

Response from the JQL search endpoint.

| Property | Type | Description |
| --- | --- | --- |
| `issues` | `JiraIssue`[] | List of issues matching the JQL query |
| `nextPageToken` | `string` (optional) | Token for the next page; absent on the last page |

## Type Definitions

### JiraAvatarUrls

Avatar URLs for a Jira user

| Property | Type | Description |
| --- | --- | --- |
| `48x48` | `string` (optional) | 48x48 pixel avatar URL |

### JiraDescription

Jira description in Atlassian Document Format (ADF)

| Property | Type | Description |
| --- | --- | --- |
| `content` | `array` (optional) | ADF content nodes - parsed to extract plain text |

### JiraIssue

A Jira issue from search results

| Property | Type | Description |
| --- | --- | --- |
| `id` | `string` | Internal Jira issue ID |
| `key` | `string` | Issue key (e.g., "PROJ-123") |
| `fields` | `JiraIssueFields` | Issue fields containing summary, status, etc. |

### JiraIssueFields

Fields of a Jira issue

| Property | Type | Description |
| --- | --- | --- |
| `summary` | `string` | Issue summary/title |
| `description` | `JiraDescription` (optional) | Issue description in ADF format |
| `issuetype` | `JiraIssueTypeRef` | Issue type (Bug, Story, Task, etc.) |
| `status` | `JiraStatusRef` | Current workflow status |
| `assignee` | `JiraUser` (optional) | Assigned user (if any) |
| `priority` | `JiraPriority` (optional) | Issue priority (if set) |
| `issuelinks` | `JiraIssueLink`[] | Links to other issues (blocks, relates, ...) |
| `parent` | `JiraLinkedIssue` (optional) | Parent issue or epic |
| `subtasks` | `JiraLinkedIssue`[] | Sub-task issues |

### JiraIssueLink

A link between two Jira issues

| Property | Type | Description |
| --- | --- | --- |
| `type` | `JiraIssueLinkType` | Link type, e.g. `Blocks` with inward text `is blocked by` |
| `inwardIssue` | `JiraLinkedIssue` (optional) | Present when the other issue is on the inward side (e.g. the blocker) |
| `outwardIssue` | `JiraLinkedIssue` (optional) | Present when the other issue is on the outward side (e.g. the blocked) |

### JiraIssueLinkType

Jira issue link type

| Property | Type | Description |
| --- | --- | --- |
| `name` | `string` | Link type name (e.g. "Blocks") |
| `inward` | `string` | Inward description (e.g. "is blocked by") |

### JiraIssueTypeRef

Reference to an issue type

| Property | Type | Description |
| --- | --- | --- |
| `id` | `string` (optional) | Issue type ID (e.g., "10001") |
| `name` | `string` | Issue type name (e.g., "Bug", "Story", "Task") |

### JiraLinkedFields

Partial fields of a related issue

| Property | Type | Description |
| --- | --- | --- |
| `issuetype` | `JiraIssueTypeRef` (optional) | Issue type of the related issue |

### JiraLinkedIssue

A related issue as embedded in links, parent and subtasks

| Property | Type | Description |
| --- | --- | --- |
| `key` | `string` | Issue key (e.g. "PROJ-12") |
| `fields` | `JiraLinkedFields` (optional) | Partial fields; only the issue type is read |

### JiraPriority

Issue priority level

| Property | Type | Description |
| --- | --- | --- |
| `name` | `string` | Priority name (e.g., "Highest", "High", "Medium", "Low", "Lowest") |

### JiraStatusRef

Reference to a workflow status

| Property | Type | Description |
| --- | --- | --- |
| `name` | `string` | Status name (e.g., "To Do", "In Progress", "Done") |

### JiraUser

Jira user information from assignable users API
GET /rest/api/3/user/assignable/search?project={key}

| Property | Type | Description |
| --- | --- | --- |
| `accountId` | `string` | Atlassian account ID (e.g., "5e3f7acd9876543210abcdef") |
| `displayName` | `string` | User's display name |
| `emailAddress` | `string` (optional) | User's email address (may be hidden by privacy settings) |
| `avatarUrls` | `JiraAvatarUrls` (optional) | Avatar URLs in various sizes |


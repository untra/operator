---
id: {{ id }}
{{#if step }}step: {{ step }}
{{/if}}status: {{ status }}
created: {{ created_datetime }}
{{#if priority }}priority: {{ priority }}
{{/if}}{{#if points }}points: {{ points }}
{{/if}}---

# Task: {{ summary }}

## Description
{{ description }}

{{#if user_story }}
## User Story
{{ user_story }}
{{/if}}

{{#if acceptance_criteria }}
## Acceptance Criteria
{{ acceptance_criteria }}
{{/if}}

## Plan
*Plan will be written to `.tickets/plans/{{ id }}.md`*

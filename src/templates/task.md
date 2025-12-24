---
id: {{ id }}
{{#if step }}step: {{ step }}
{{/if}}{{#if project }}project: {{ project }}
{{/if}}status: {{ status }}
created: {{ created_datetime }}
{{#if priority }}priority: {{ priority }}
{{/if}}---

# Task: {{ summary }}

{{#if context }}
## Context
{{ context }}
{{/if}}

{{#if acceptance_criteria }}
## Acceptance Criteria
{{ acceptance_criteria }}
{{/if}}

## Plan
*Plan will be written to `.tickets/plans/{{ id }}.md`*

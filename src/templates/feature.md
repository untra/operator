---
id: {{ id }}
{{#if step }}step: {{ step }}
{{/if}}{{#if project }}project: {{ project }}
{{/if}}status: {{ status }}
created: {{ created_datetime }}
branch: {{ branch }}
{{#if priority }}priority: {{ priority }}
{{/if}}---

# Feature: {{ summary }}

{{#if context }}
## Context
{{ context }}
{{/if}}

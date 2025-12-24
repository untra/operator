---
id: {{ id }}
{{#if step }}step: {{ step }}
{{/if}}{{#if project }}project: {{ project }}
{{/if}}status: {{ status }}
created: {{ created_datetime }}
branch: {{ branch }}
{{#if priority }}priority: {{ priority }}
{{/if}}{{#if severity }}severity: {{ severity }}
{{/if}}{{#if fix_type }}fix_type: {{ fix_type }}
{{/if}}{{#if parent }}parent: {{ parent }}
{{/if}}---

# Fix: {{ summary }}

{{#if context }}
## Context
{{ context }}
{{/if}}

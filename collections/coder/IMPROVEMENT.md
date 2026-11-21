---
id: {{ id }}
{{#if step }}step: {{ step }}
{{/if}}status: {{ status }}
created: {{ created_datetime }}
branch: {{ branch }}
{{#if priority }}priority: {{ priority }}
{{/if}}---

# Improvement: {{ summary }}

{{#if motivation }}
## Motivation
{{ motivation }}
{{/if}}
{{#if customer }}
## Customer
{{ customer }}
{{/if}}

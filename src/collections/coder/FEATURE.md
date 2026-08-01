---
id: {{ id }}
{{#if step }}step: {{ step }}
{{/if}}status: {{ status }}
created: {{ created_datetime }}
branch: {{ branch }}
{{#if priority }}priority: {{ priority }}
{{/if}}---

# Feature: {{ summary }}

{{#if user_story }}
## User Story
{{ user_story }}
{{/if}}
{{#if customer }}
## Customer
{{ customer }}
{{/if}}

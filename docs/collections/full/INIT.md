---
id: {{ id }}
{{#if step }}step: {{ step }}
{{/if}}workspace: {{ workspace }}
{{#if branding_name }}branding_name: {{ branding_name }}
{{/if}}status: {{ status }}
created: {{ created_datetime }}
---

# Workspace Init: {{ summary }}

## Workspace
{{ workspace }}

{{#if branding_name }}
## Branding
{{ branding_name }}
{{/if}}

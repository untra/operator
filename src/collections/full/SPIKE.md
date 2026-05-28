---
id: {{ id }}
{{#if step }}step: {{ step }}
{{/if}}{{#if scope }}scope: {{ scope }}
{{/if}}status: {{ status }}
created: {{ created_datetime }}
{{#if priority }}priority: {{ priority }}
{{/if}}---

# Spike: {{ summary }}

{{#if context }}
## Context
{{ context }}
{{/if}}

{{#if success_criteria }}
## Success Criteria
{{ success_criteria }}
{{/if}}

## Conversation Log
### Session: {{ created_date }}

## Findings
<!-- Document discoveries here -->

---
id: {{ id }}
{{#if step }}step: {{ step }}
{{/if}}{{#if project }}project: {{ project }}
{{/if}}{{#if scope }}scope: {{ scope }}
{{/if}}status: {{ status }}
created: {{ created_datetime }}
{{#if severity }}severity: {{ severity }}
{{/if}}{{#if source }}source: {{ source }}
{{/if}}---

# Investigation: {{ summary }}

{{#if observed_behavior }}
## Observed Behavior
{{ observed_behavior }}
{{/if}}

{{#if expected_behavior }}
## Expected Behavior
{{ expected_behavior }}
{{/if}}

{{#if impact }}
## Impact
{{ impact }}
{{/if}}

## Timeline
| Time | Event |
|------|-------|
| {{ created_datetime }} | Investigation opened |

## Findings
<!-- Document investigation progress here -->

---
id: {{ id }}
{{#if step }}step: {{ step }}
{{/if}}status: {{ status }}
created: {{ created_datetime }}
branch: {{ branch }}
{{#if priority }}priority: {{ priority }}
{{/if}}---

# Bug: {{ summary }}

{{#if current_behavior }}
## Current Behavior
{{ current_behavior }}
{{/if}}
{{#if expected_behavior }}
## Expected Behavior
{{ expected_behavior }}
{{/if}}
{{#if steps_to_reproduce }}
## Steps to Reproduce
{{ steps_to_reproduce }}
{{/if}}

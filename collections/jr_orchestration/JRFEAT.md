---
id: {{ id }}
{{#if step }}step: {{ step }}
{{/if}}status: {{ status }}
created: {{ created_datetime }}
project: {{ project }}
branch: {{ branch }}
{{#if parent_plan }}parent_plan: {{ parent_plan }}
{{/if}}---

# JR Feature: {{ summary }}

## Task Chain
{{ task_chain }}

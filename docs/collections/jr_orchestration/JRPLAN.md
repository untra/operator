---
id: {{ id }}
{{#if step }}step: {{ step }}
{{/if}}status: {{ status }}
created: {{ created_datetime }}
project: {{ project }}
{{#if priority }}priority: {{ priority }}
{{/if}}---

# JR Plan: {{ summary }}

## Plan Source
{{ plan_source }}

## Review Policy
{{ review_policy }}

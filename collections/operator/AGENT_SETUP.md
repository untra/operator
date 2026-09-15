---
id: {{ id }}
{{#if step }}step: {{ step }}
{{/if}}status: {{ status }}
created: {{ created_datetime }}
{{#if agent_tool }}agent_tool: {{ agent_tool }}
{{/if}}---

# Agent Setup: {{ summary }}

{{#if agent_tool }}
## Target Agent
{{ agent_tool }}
{{/if}}

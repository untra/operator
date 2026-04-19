# Polymorphic Step Types — Remaining Work Handoff

## What's Done (Phases 1-3 + Phase 4 partial)

All schema types, validation, step output artifacts, single-agent executors, and multi-agent aggregation functions are implemented and tested. 1,666 tests pass, `cargo fmt && cargo clippy -- -D warnings` clean.

### Completed artifacts:

| File | What was added |
|------|---------------|
| `src/templates/schema.rs` | `StepTypeTag` enum (8 variants), `ClassifierConfig`, `ClassifierOutputType`, `RagConfig`, `RagSource`, `DelegatorStepConfig`, `McpStepConfig`, `McpToolRef`, `MultiModelConfig`, `VotingStrategy`, `VotingMode`, `MultiPromptConfig`, `SelectionStrategy`, `MatrixedConfig`, `MatrixedOutputFormat`. `StepSchema` updated with `step_type` field + 7 optional `*_config` fields. `validate_type_config()` on StepSchema. |
| `src/templates/step_type.rs` | `classifier_json_schema()`, `prompt_augmentation()`, `effective_allowed_tools()`, `effective_agent()`, `aggregate_multi_model()`, `aggregate_multi_prompt()`, `aggregate_matrixed()`, `apply_votes()`, `apply_selection()`, `apply_aggregation()`, `select_winner_by_strategy()`. 24 unit tests. |
| `src/steps/manager.rs` | `load_step_outputs()` reads `{worktree}/.tickets/steps/{name}.output.json` into Handlebars context as `{{ steps.{name}.* }}`. `render_prompt()` injects step outputs. 5 tests. |
| `src/steps/session.rs` | `generate_prompt()` uses `effective_allowed_tools()` and `prompt_augmentation()`. |
| `src/agents/launcher/step_config.rs` | `get_step_config()` derives `json_schema` from classifier config, injects MCP server permissions, merges delegator permissions. |
| `src/agents/agent_switcher.rs` | `needs_switch()` uses `effective_agent()`. |
| `src/agents/launcher/mod.rs` | `collect_tools_for_ticket()` uses `effective_agent()`. |
| `src/queue/ticket.rs` | `advance_step()` uses `effective_agent()`. |
| `src/state.rs` | `MultiAgentGroup`, `MultiAgentPhase`, `multi_agent_groups` field on `State`, helper methods: `create_multi_agent_group()`, `get_group_for_agent()`, `get_group_for_ticket()`, `record_agent_output()`, `update_group_phase()`, `complete_group()`, `cleanup_finished_groups()`. |
| `src/docs_gen/issuetype_json_schema.rs` | Generator now outputs to `src/schemas/issuetype_schema.json` (overwrites stale hand-written file). |
| `src/schemas/issuetype_schema.json` | Auto-generated, includes all new step types. |

---

## Remaining Work

### Phase 4d: Fan-out launch in `src/agents/launcher/mod.rs`

**Where to insert**: In `launch_with_options()` (line 227), after getting the effective step (around line 247), detect if the current step is a multi-agent type and branch:

```rust
// After getting the step schema for the ticket:
let step = ticket.current_step_schema();
if let Some(ref step) = step {
    match step.step_type {
        StepTypeTag::MultiModel => return self.launch_multi_model(ticket, step, options).await,
        StepTypeTag::MultiPrompt => return self.launch_multi_prompt(ticket, step, options).await,
        StepTypeTag::Matrixed => return self.launch_matrixed(ticket, step, options).await,
        _ => {} // fall through to existing single-agent launch
    }
}
```

**New methods to add on `Launcher`**:

1. `launch_multi_model()`:
   - Read `step.multi_model_config.delegators` (e.g., `["claude-opus", "gemini-pro"]`)
   - For each delegator name, resolve to a `Delegator` from `config.delegators`
   - Call `launch_with_options()` (or the inner launch_in_tmux/cmux) N times, each with:
     - `session_name = format!("op-{}-{}", ticket.id, delegator_name)`
     - The delegator's tool+model
     - Same prompt (from the step)
     - Same worktree
   - Register each sub-agent via `state.add_agent_with_options()`
   - Create a `MultiAgentGroup` via `state.create_multi_agent_group(ticket_id, step_name, "multi_model", agent_ids)`
   - Return the group_id (or first agent_id)

2. `launch_multi_prompt()`:
   - Read `step.multi_prompt_config.prompt_variations`
   - Same delegator for all (from `multi_prompt_config.agent` or default)
   - N launches, each with a different prompt from `prompt_variations[i]`
   - Session names: `format!("op-{}-v{}", ticket.id, i)`
   - Create group with keys `"0"`, `"1"`, ... for `individual_outputs`

3. `launch_matrixed()`:
   - NxM launches: delegators × prompt_variations
   - Session names: `format!("op-{}-{}-v{}", ticket.id, delegator_name, prompt_idx)`
   - Create group with keys `"{delegator}:{prompt_idx}"`

**Important**: Each sub-agent writes its output to `{worktree}/.tickets/steps/{step_name}/{agent_id_or_key}.json`. The sub-agent's prompt should include an instruction like: "Write your final output to `.tickets/steps/{step_name}/{key}.json`."

**Slot accounting**: Check `state.running_agents().len() + N <= config.effective_max_agents()` before launching. If not enough slots, either fail with an error or launch partial (queuing is complex — fail-fast is simpler for v1).

### Phase 4e: Group-aware sync in `src/agents/sync.rs`

**Where to insert**: In `sync_all()` (line 99), at the `SyncAction::StepCompleted` handler (line 249):

```rust
SyncAction::StepCompleted => {
    // Check if this agent belongs to a multi-agent group
    if let Some(group) = state.get_group_for_agent(&agent_id) {
        let group_id = group.group_id.clone();
        let step_type = group.step_type.clone();
        let step_name = group.step_name.clone();

        // Read this agent's output from worktree
        let output = read_agent_step_output(ticket, &step_name, &agent_id);

        // Record it; returns true if all agents in group are done
        let all_done = state.record_agent_output(&agent_id, output)?;

        if all_done {
            let group = state.get_group_for_ticket(&ticket.id).unwrap().clone();

            // Get step schema for config access
            let step_schema = ticket.current_step_schema();

            let aggregated = match step_type.as_str() {
                "multi_model" => {
                    let config = step_schema.and_then(|s| s.multi_model_config.clone());
                    if let Some(cfg) = config {
                        let mut result = step_type::aggregate_multi_model(
                            &group.individual_outputs, &cfg
                        );
                        // TODO: Phase 2 voting round if cfg.share_answers
                        // For v1, use the pre-vote winner selection
                        result
                    } else { serde_json::json!(null) }
                }
                "multi_prompt" => { /* similar with aggregate_multi_prompt */ }
                "matrixed" => { /* similar with aggregate_matrixed */ }
                _ => serde_json::json!(null),
            };

            // Write aggregated output artifact
            write_step_output_artifact(ticket, &step_name, &aggregated)?;

            // Mark group complete
            state.complete_group(&group_id, aggregated)?;

            // Advance the ticket's step (single advance for the whole group)
            ticket.advance_step()?;

            // Clean up sub-agent records
            for aid in &group.agent_ids {
                state.remove_agent(aid)?;
            }
        }
        // else: not all done yet, wait for remaining sub-agents
    } else {
        // Existing single-agent completion logic (unchanged)
        match ticket.advance_step() { ... }
    }
}
```

**Helper functions to add**:

```rust
fn read_agent_step_output(ticket: &Ticket, step_name: &str, key: &str) -> serde_json::Value {
    let worktree = ticket.worktree_path.as_deref().unwrap_or(".");
    let path = format!("{worktree}/.tickets/steps/{step_name}/{key}.json");
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(serde_json::Value::Null)
}

fn write_step_output_artifact(
    ticket: &Ticket, step_name: &str, output: &serde_json::Value
) -> Result<()> {
    let worktree = ticket.worktree_path.as_deref().unwrap_or(".");
    let dir = format!("{worktree}/.tickets/steps");
    std::fs::create_dir_all(&dir)?;
    let path = format!("{dir}/{step_name}.output.json");
    std::fs::write(&path, serde_json::to_string_pretty(output)?)?;
    Ok(())
}
```

### Phase 4f: REST API for grouped completion in `src/rest/routes/launch.rs`

**Where to insert**: In `complete_step()` (line 173), after determining the status:

```rust
// After existing status determination (line 204-225):

// Check if this agent is part of a multi-agent group
let api_state = state.lock().await;
if let Some(group) = api_state.get_group_for_agent(&request.session_id.unwrap_or_default()) {
    // This is a sub-agent completion — collect but don't advance
    let output = request.output.as_ref()
        .and_then(|o| o.summary.as_ref())
        .map(|s| serde_json::json!(s))
        .unwrap_or(serde_json::Value::Null);

    let all_done = api_state.record_agent_output(&agent_id, output)?;

    if all_done {
        // Return status indicating the group is ready for aggregation
        // The sync loop will handle the actual aggregation
        return Ok(Json(StepCompleteResponse {
            status: "group_complete".to_string(),
            auto_proceed: false,  // sync loop handles advancement
            ..default_response
        }));
    } else {
        return Ok(Json(StepCompleteResponse {
            status: "group_partial".to_string(),
            auto_proceed: false,
            ..default_response
        }));
    }
}
// else: fall through to existing single-agent logic
```

Add `"group_complete"` and `"group_partial"` as recognized status values.

### Phase 2 Voting/Selection Rounds

The most complex part. When `all_done` is true in the sync loop and the step type is `multi_model` with `share_answers: true`:

1. **Single Judge mode** (`voting_mode == SingleJudge`):
   - Launch ONE new agent (first delegator) with a voting prompt that includes all collected responses
   - Use `classifier_json_schema` to generate a schema for `{ "vote": <int>, "reasoning": "<string>" }`
   - When this agent completes, call `apply_votes()` on the aggregated output
   - Then write the artifact and advance

2. **Multi Voter mode** (`voting_mode == MultiVoter`):
   - Transition group phase to `Voting`
   - Launch N new agents (one per original delegator) with voting prompts
   - Each votes via structured output
   - When all N voting agents complete, tally and `apply_votes()`

For v1, implement `SingleJudge` only. `MultiVoter` can be a follow-up.

For `multi_prompt` with `selection_strategy`:
   - Launch one agent with selection_prompt + all variation outputs
   - Agent returns `{ "selected_index": N }`
   - Call `apply_selection()` on the aggregated output

---

## Phase 5: Collection Examples & Remaining Items

### 5a. Add `if/then` conditionals to JSON schema

In `src/docs_gen/issuetype_json_schema.rs`, in `generate()` (after line 48 where schema metadata is added), add post-processing:

```rust
// Add if/then conditionals for step type validation
if let Some(defs) = schema_value.get("$defs").or(schema_value.get("definitions")) {
    if defs.get("StepSchema").is_some() {
        let step_types = [
            ("classifier", "classifier_config"),
            ("rag", "rag_config"),
            ("delegator", "delegator_config"),
            ("mcp", "mcp_config"),
            ("multi_model", "multi_model_config"),
            ("multi_prompt", "multi_prompt_config"),
            ("matrixed", "matrixed_config"),
        ];
        // Find StepSchema in $defs and add allOf with if/then blocks
        // Each: if { properties: { type: { const: X } } } then { required: [X_config] }
    }
}
```

This is optional — the Rust-side `validate_type_config()` already enforces this at load time.

### 5b. Re-enable JSON_SCHEMA_ENABLED for classifier steps

In `src/agents/launcher/llm_command.rs`:
- Line 13: Change `const JSON_SCHEMA_ENABLED: bool = false;` to `true`
- OR: Make it conditional — only enable for classifier steps by checking `step_config.json_schema.is_some()` regardless of the constant

The safer approach: remove the constant entirely and always pass `--json-schema` when `step_config.json_schema` is `Some`. The original issue was command-line length, but we write schemas to files (not inline), so the path length should be fine.

### 5c. Regenerate documentation

```bash
cargo run -- docs --only issuetype-json-schema  # Regenerates src/schemas/issuetype_schema.json
cargo run -- docs --only issuetype              # Regenerates docs/schemas/issuetype.md
```

### 5d. Create example collection with new step types

Create `src/collections/advanced/` with example issuetypes demonstrating each new step type:

1. **`REVIEW.json`** — multi-model consensus review:
   ```json
   {
     "key": "REVIEW",
     "name": "Multi-Model Review",
     "steps": [
       {
         "name": "review",
         "type": "multi_model",
         "prompt": "Review this PR for issues",
         "outputs": ["review"],
         "multi_model_config": {
           "delegators": ["claude-opus", "gemini-pro"],
           "voting_strategy": "majority",
           "share_answers": true
         },
         "next_step": "apply"
       },
       {
         "name": "apply",
         "type": "task",
         "prompt": "Apply the winning review: {{ steps.review.winner_response }}",
         "outputs": ["code"],
         "allowed_tools": ["Read", "Write", "Edit"]
       }
     ]
   }
   ```

2. **`ASSESS.json`** — classifier + RAG pipeline:
   ```json
   {
     "steps": [
       { "type": "rag", "name": "gather", ... },
       { "type": "classifier", "name": "classify", "classifier_config": { "output_type": "enum", "options": [...] } },
       { "type": "task", "name": "act", "prompt": "Severity is {{ steps.classify.value }}" }
     ]
   }
   ```

### 5e. Add `advanced` collection preset

In `src/config.rs`, add to `CollectionPreset`:
```rust
pub enum CollectionPreset {
    Simple,
    DevKanban,
    DevopsKanban,
    Advanced,   // NEW
    Custom,
}
```

With `issue_types()` returning the advanced collection types.

---

## Testing Checklist

### Unit tests (already passing — 1,666 total):
- [x] `StepTypeTag` serde round-trip for all 8 variants
- [x] All config structs deserialize from JSON
- [x] `validate_type_config()` catches missing configs, invalid options, minimum counts
- [x] `classifier_json_schema()` generates correct schema for all 5 output types
- [x] `prompt_augmentation()` produces correct text for each step type
- [x] `effective_agent()` resolves from type-specific configs with fallback
- [x] `effective_allowed_tools()` resolves from type-specific configs with fallback
- [x] `load_step_outputs()` reads artifacts into Handlebars context
- [x] `render_prompt()` interpolates `{{ steps.X.value }}` correctly
- [x] `aggregate_multi_model()` collects responses and selects winner
- [x] `apply_votes()` updates winner based on vote tallies
- [x] `aggregate_multi_prompt()` collects variations
- [x] `apply_selection()` updates selected response
- [x] `aggregate_matrixed()` builds NxM matrix
- [x] `apply_aggregation()` sets aggregated result
- [x] `MultiAgentGroup` state persistence (serde round-trip via `#[serde(default)]`)
- [x] All existing collection JSONs parse without errors (backward compat)

### Tests to write for Phase 4d-4f:
- [ ] `launch_multi_model()` creates N agents + 1 group in state
- [ ] `launch_multi_prompt()` creates N agents with different prompts
- [ ] Slot limit check: launching exceeding `max_parallel` fails gracefully
- [ ] Sync: single sub-agent completion → group not done yet
- [ ] Sync: all sub-agents complete → triggers aggregation + advance_step
- [ ] Sync: aggregated output written to `.tickets/steps/{name}.output.json`
- [ ] REST: `complete_step` returns `group_partial` for incomplete groups
- [ ] REST: `complete_step` returns `group_complete` when last agent finishes
- [ ] Voting round (single_judge): launches judge agent, processes vote output
- [ ] State cleanup: `cleanup_finished_groups()` removes completed groups

### CI validation:
```bash
cargo fmt                      # Must be clean
cargo clippy -- -D warnings    # Must be clean
cargo test                     # All tests pass
cargo run -- docs              # Regenerates all docs without error
```

### Manual E2E test:
1. Configure 2+ delegators in operator config
2. Create a ticket with a `multi_model` step referencing those delegators
3. Launch the ticket — verify N tmux sessions spawn
4. Let all agents complete — verify aggregated output artifact appears
5. Verify the next step sees `{{ steps.{name}.winner_response }}`

---

## Key Files Reference

| File | Purpose |
|------|---------|
| `src/templates/schema.rs` | All type definitions (StepTypeTag, configs, enums) |
| `src/templates/step_type.rs` | Prompt augmentation, effective_agent/tools, aggregation functions |
| `src/steps/manager.rs` | Step output artifact loading into Handlebars context |
| `src/steps/session.rs` | Prompt generation with type augmentation |
| `src/agents/launcher/step_config.rs` | Step config extraction (classifier JSON schema, MCP injection) |
| `src/agents/launcher/mod.rs` | **MODIFY**: Add fan-out launch methods |
| `src/agents/sync.rs` | **MODIFY**: Add group-aware completion detection |
| `src/rest/routes/launch.rs` | **MODIFY**: Add grouped completion handling |
| `src/state.rs` | MultiAgentGroup tracking + helper methods |
| `src/schemas/issuetype_schema.json` | Auto-generated JSON schema (run `cargo run -- docs --only issuetype-json-schema`) |
| `src/docs_gen/issuetype_json_schema.rs` | Schema generator (outputs to `src/schemas/`) |
| `src/agents/launcher/llm_command.rs:13` | `JSON_SCHEMA_ENABLED` constant — set to `true` |

## Branch

All work is on branch `issuetype-onboarding-kanban-both`. No commits have been made for this work yet — all changes are unstaged.

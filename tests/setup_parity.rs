//! Setup wizard parity tests.
//!
//! The wizard's step catalog (`src/startup/steps.rs`) is the single source of
//! truth for the ratatui renderer, the generated TypeScript binding and the
//! hosted docs page. These tests keep the three in step.
//!
//! Uses `include_str!` to scan source files (same pattern as
//! `surface_parity.rs`) because `startup` is a crate-private module and the
//! wizard itself is bin-only, so neither is reachable from an integration test.

const STEPS_RS: &str = include_str!("../src/startup/steps.rs");
const BINDING_TS: &str = include_str!("../bindings/SetupStep.ts");
const STARTUP_DOC: &str = include_str!("../docs/startup/index.md");
const SETUP_MOD_RS: &str = include_str!("../src/ui/setup/mod.rs");
const GIT_STEP_RS: &str = include_str!("../src/ui/setup/steps/git.rs");
const MODEL_STEP_RS: &str = include_str!("../src/ui/setup/steps/model_server.rs");
const WEB_STEPS_TSX: &str = include_str!("../ui/src/routes/onboarding/steps.tsx");

/// The assertions below scrape TSX source, so collapse what the formatter is
/// free to rewrite: quote style and line wrapping. Prose keeps single spaces.
fn web_steps_tsx() -> String {
    WEB_STEPS_TSX
        .replace('\'', "\"")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Same, for code shapes: the formatter may break a call chain across lines, so
/// compare with all whitespace removed.
fn tsx_contains_code(needle: &str) -> bool {
    fn compact(s: &str) -> String {
        s.chars().filter(|c| !c.is_whitespace()).collect()
    }
    compact(&WEB_STEPS_TSX.replace('\'', "\"")).contains(&compact(needle))
}

/// Variant names in `SetupStep::ALL`, in declaration order.
fn catalog_order() -> Vec<String> {
    let all = STEPS_RS
        .split_once("pub const ALL:")
        .expect("steps.rs must declare SetupStep::ALL")
        .1;
    let body = all.split_once('[').unwrap().1.split_once("];").unwrap().0;
    body.lines()
        .filter_map(|l| l.trim().strip_prefix("SetupStep::"))
        .map(|v| v.trim_end_matches(',').to_string())
        .collect()
}

/// Slugs in the order `slug()` matches them.
fn catalog_slugs() -> Vec<String> {
    let arm = STEPS_RS
        .split_once("pub fn slug(self)")
        .expect("steps.rs must define slug()")
        .1;
    let body = arm.split_once('{').unwrap().1;
    body.lines()
        .filter_map(|l| l.trim().split_once("=> \""))
        .map(|(_, rest)| rest.split('"').next().unwrap_or("").to_string())
        .take_while(|s| !s.is_empty())
        .collect()
}

/// Step display names from the generated docs page headings.
fn doc_step_names() -> Vec<String> {
    STARTUP_DOC
        .lines()
        .filter_map(|l| l.strip_prefix("### "))
        .filter_map(|h| h.split_once(". "))
        .map(|(_, name)| name.trim().to_string())
        .collect()
}

/// `name:` literals from the `info()` match, in declaration order.
fn catalog_names() -> Vec<String> {
    let arm = STEPS_RS
        .split_once("pub fn info(self)")
        .expect("steps.rs must define info()")
        .1;
    arm.lines()
        .filter_map(|l| l.trim().strip_prefix("name: \""))
        .map(|rest| rest.split('"').next().unwrap_or("").to_string())
        .collect()
}

#[test]
fn test_catalog_is_non_empty() {
    assert!(
        !catalog_order().is_empty(),
        "failed to parse SetupStep::ALL"
    );
    assert_eq!(catalog_order().len(), catalog_slugs().len());
    assert_eq!(catalog_order().len(), catalog_names().len());
}

/// Defect E regression: the generated docs page drifted out of the wizard's
/// real order and nothing caught it, because the only guard was a step count.
#[test]
fn test_docs_page_lists_steps_in_catalog_order() {
    assert_eq!(
        doc_step_names(),
        catalog_names(),
        "docs/startup/index.md is stale - run `cargo run -- docs --only startup`"
    );
}

#[test]
fn test_binding_union_matches_catalog_slugs() {
    let union = BINDING_TS
        .split_once("export type SetupStep =")
        .expect("binding must declare SetupStep")
        .1;
    let members: Vec<String> = union
        .split('|')
        .map(|m| m.trim().trim_end_matches(';').trim_matches('"').to_string())
        .collect();
    assert_eq!(
        members,
        catalog_slugs(),
        "bindings/SetupStep.ts is stale - run `make bindings`"
    );
}

/// Every variant must be reachable from the wizard state machine; a step the
/// renderer never selects is dead code (defect A's failure mode).
#[test]
fn test_every_catalog_step_is_referenced_by_the_wizard() {
    for variant in catalog_order() {
        assert!(
            SETUP_MOD_RS.contains(&format!("SetupStep::{variant}")),
            "SetupStep::{variant} is never referenced in src/ui/setup/mod.rs"
        );
    }
}

/// Provider slugs the wizard must never hardcode: the offered set comes from
/// `integrations::catalog::onboardable`, so promoting a provider into
/// onboarding is a `SupportStatus` bump rather than an edit in each surface.
/// The same list will back the web wizard over REST.
#[test]
fn test_wizard_steps_do_not_hardcode_provider_slugs() {
    const SLUGS: &[&str] = &[
        "github",
        "gitlab",
        "gitea",
        "bitbucket",
        "forgejo",
        "anthropic-api",
        "openai-api",
        "google-api",
        "ollama",
        "openrouter",
        "openai-compat",
        "lmstudio",
    ];
    for (name, source) in [
        ("steps/git.rs", GIT_STEP_RS),
        ("steps/model_server.rs", MODEL_STEP_RS),
    ] {
        for slug in SLUGS {
            assert!(
                !source.contains(&format!("\"{slug}\"")),
                "{name} hardcodes the provider slug {slug:?}; derive the list from \
                 integrations::catalog::onboardable instead"
            );
        }
    }
}

/// The wizard must key its provider rows off the catalog, not a per-vertical
/// enum - the enums include Proto entries that onboarding does not advertise.
#[test]
fn test_wizard_derives_provider_lists_from_the_catalog() {
    assert!(
        SETUP_MOD_RS.contains("onboardable(Vertical::Git)"),
        "git rows must come from the catalog"
    );
    assert!(
        SETUP_MOD_RS.contains("onboardable(Vertical::Model)"),
        "model rows must come from the catalog"
    );
}

#[test]
fn test_web_wizard_has_an_exhaustive_component_map() {
    assert!(
        tsx_contains_code("satisfies Record<SetupStep, StepComponent>"),
        "the web wizard must fail TypeScript compilation when the Rust step union grows"
    );
    for slug in catalog_slugs() {
        assert!(
            tsx_contains_code(&format!("{slug}:")) || tsx_contains_code(&format!("\"{slug}\":")),
            "web component map is missing {slug:?}"
        );
    }
}

#[test]
fn test_web_wizard_derives_provider_lists_from_rest_catalogs() {
    assert!(tsx_contains_code("entry.vertical === \"model\""));
    assert!(tsx_contains_code("api.gitProviders()"));
    assert!(tsx_contains_code("api.kanbanProviders()"));
}

#[test]
fn test_web_parity_scope_cuts_are_explicit() {
    assert!(web_steps_tsx().contains("Ticket creation is read-only"));
    assert!(tsx_contains_code("wrapperSteps.has(step)"));
}

/// Coder template parameters are held in `draft.coderParameters` (stable row ids)
/// and only folded into the request payload at submit. Editing the target
/// name or template must not clear them - the earlier version of these handlers
/// wrote `parameters: {}` back into the target on every keystroke.
#[test]
fn test_coder_parameter_rows_survive_target_name_and_template_edits() {
    assert!(
        tsx_contains_code("coderParameters: current.coderParameters.map("),
        "parameter rows must be edited in place by id, not rebuilt from the target"
    );
    let carried = WEB_STEPS_TSX
        .matches("? current.executionTarget.parameters")
        .count();
    assert_eq!(
        carried, 2,
        "both the target-name and the template handler must carry existing parameters \
         through an edit instead of resetting them to an empty map"
    );

    // The only legitimate empty initialisation is selecting the coder kind for
    // the first time; every other site would silently drop entered parameters.
    let reset = WEB_STEPS_TSX.matches("parameters: {},").count();
    assert_eq!(
        reset, 1,
        "only the coder-kind selection may initialise parameters to an empty map"
    );
}

/// Parameter values can carry credentials-adjacent template input. The review
/// step acknowledges that parameters exist and where they land, but never renders a value.
#[test]
fn test_confirm_step_reports_parameter_count_without_values() {
    let confirm = WEB_STEPS_TSX
        .split_once("const Confirm: StepComponent")
        .expect("steps.tsx must define a Confirm step")
        .1
        .split_once("export const STEP_COMPONENTS")
        .expect("Confirm must precede the component map")
        .0;

    assert!(
        confirm.contains("draft.coderParameters.length"),
        "the review summary must report how many Coder parameters were entered"
    );
    assert!(
        confirm.contains("stored in the project configuration"),
        "the review summary must say where parameter values are persisted"
    );
    assert!(
        !confirm.contains("parameter.value") && !confirm.contains(".value}"),
        "the review summary must never render a Coder parameter value"
    );
}

/// Empty and duplicate names are rejected before submit; values are sent verbatim
#[test]
fn test_web_wizard_validates_coder_parameter_names() {
    const PAGE_TSX: &str = include_str!("../ui/src/routes/onboarding/OnboardingPage.tsx");

    assert!(
        PAGE_TSX.contains("Coder parameter names cannot be empty."),
        "the wizard must reject an empty parameter name"
    );
    assert!(
        PAGE_TSX.contains("Coder parameter names must be unique."),
        "the wizard must reject duplicate parameter names"
    );
    assert!(
        PAGE_TSX.contains("[name.trim(), value]"),
        "parameter names are trimmed but values must be submitted verbatim"
    );
}

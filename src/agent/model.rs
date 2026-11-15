//! The model of a session: zendo names the agent sessions with a light model (docs/zendo/
//! session-names/session-names.md, NAMES-12), through the config options that the agent gives for a session.

use agent_client_protocol::v1::schema::{SessionConfigKind, SessionConfigOption, SessionConfigOptionCategory, SessionConfigSelectOption, SessionConfigSelectOptions};

/// The option of the model and its value whose ID or name holds `model`, without regard to case;
/// an exact value first. None: the agent has no such model (NAMES-13).
pub fn choose(options: &[SessionConfigOption], model: &str) -> Option<(String, String)> {
    let want = model.trim().to_lowercase();
    let option = options.iter().find(|o| matches!(o.category, Some(SessionConfigOptionCategory::Model)) || o.id.0.as_ref() == "model")?;
    let SessionConfigKind::Select(select) = &option.kind else { return None };
    let values: Vec<&SessionConfigSelectOption> = match &select.options {
        SessionConfigSelectOptions::Ungrouped(v) => v.iter().collect(),
        SessionConfigSelectOptions::Grouped(g) => g.iter().flat_map(|g| g.options.iter()).collect(),
        _ => Vec::new(),
    };
    let id = |v: &&SessionConfigSelectOption| v.value.0.to_lowercase();
    let exact = values.iter().find(|v| id(v) == want);
    let holds = || values.iter().find(|v| id(v).contains(&want) || v.name.to_lowercase().contains(&want));
    exact.or_else(holds).map(|v| (option.id.0.to_string(), v.value.0.to_string()))
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;

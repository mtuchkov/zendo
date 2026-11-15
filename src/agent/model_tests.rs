use super::*;
use agent_client_protocol::v1::schema::{SessionConfigOptionCategory, SessionConfigSelectOption};

fn models(values: &[(&str, &str)]) -> Vec<SessionConfigOption> {
    let options: Vec<SessionConfigSelectOption> = values.iter().map(|(v, n)| SessionConfigSelectOption::new(v.to_string(), n.to_string())).collect();
    vec![
        SessionConfigOption::select("mode", "Mode", "default", vec![SessionConfigSelectOption::new("haiku-mode", "Haiku mode")]),
        SessionConfigOption::select("model", "Model", "default", options).category(SessionConfigOptionCategory::Model),
    ]
}

// NAMES-12: the value of the model option whose id or name holds the name of the config.
#[test]
fn choose_finds_the_model_by_its_id_or_name() {
    let opts = models(&[("default", "Default (Opus)"), ("claude-haiku-4-5", "Haiku"), ("sonnet", "Sonnet")]);
    assert_eq!(choose(&opts, "haiku"), Some(("model".to_string(), "claude-haiku-4-5".to_string())));
    assert_eq!(choose(&opts, "Sonnet"), Some(("model".to_string(), "sonnet".to_string())));
    assert_eq!(choose(&opts, "luna"), None, "NAMES-13: no such model");
}

// NAMES-12: an exact value wins over one that only holds the name.
#[test]
fn choose_prefers_an_exact_value() {
    let opts = models(&[("gpt-luna-pro", "Luna Pro"), ("luna", "Luna")]);
    assert_eq!(choose(&opts, "luna"), Some(("model".to_string(), "luna".to_string())));
}

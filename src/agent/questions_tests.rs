use super::*;

/// The form of the Claude adapter for one question and its own answer (AskUserQuestion).
fn form(json: serde_json::Value) -> ElicitationSchema {
    serde_json::from_value(json).unwrap()
}

// ACT-4: a single question takes its text from the message, its choices from `oneOf`, and its own
// answer from the free field beside it.
#[test]
fn a_form_is_questions_with_choices() {
    let s = form(serde_json::json!({
        "type": "object",
        "properties": {
            "question_0": {"type": "string", "title": "DB", "oneOf": [{"const": "pg", "title": "Postgres"}, {"const": "lite", "title": "SQLite"}]},
            "question_0_custom": {"type": "string", "title": "Other"}
        }
    }));
    let q = questions_of("Which database?", &s);
    assert_eq!(q.len(), 1);
    assert_eq!((q[0].text.as_str(), q[0].many, q[0].other.as_deref()), ("Which database?", false, Some("question_0_custom")));
    assert_eq!(q[0].choices, vec![("pg".to_string(), "Postgres".to_string()), ("lite".into(), "SQLite".into())]);
}

// ACT-4: with more questions, each takes its text from its description; an array is many choices.
#[test]
fn more_questions_and_many_choices() {
    let s = form(serde_json::json!({
        "type": "object",
        "properties": {
            "question_0": {"type": "string", "description": "Which?", "oneOf": [{"const": "a", "title": "A"}]},
            "question_1": {"type": "array", "description": "Which ones?", "items": {"anyOf": [{"const": "x", "title": "X"}, {"const": "y", "title": "Y"}]}}
        }
    }));
    let q = questions_of("Please answer the following questions.", &s);
    assert_eq!(q.iter().map(|q| (q.text.as_str(), q.many)).collect::<Vec<_>>(), vec![("Which?", false), ("Which ones?", true)]);
}

// ACT-4: the picks of the user are the content of the form: one value, many values, own text.
#[test]
fn the_picks_are_the_content() {
    let qs = vec![
        Question { field: "question_0".into(), choices: vec![("pg".into(), "Postgres".into())], other: Some("question_0_custom".into()), ..Default::default() },
        Question { field: "question_1".into(), many: true, choices: vec![("x".into(), "X".into())], ..Default::default() },
    ];
    let picks = vec![Pick { values: vec![], own: "MySQL".into() }, Pick { values: vec!["x".into()], own: String::new() }];
    let c = content(&qs, &picks);
    assert_eq!(c.get("question_0_custom"), Some(&ElicitationContentValue::String("MySQL".into())));
    assert_eq!(c.get("question_1"), Some(&ElicitationContentValue::StringArray(vec!["x".into()])));
    assert_eq!(c.get("question_0"), None);
}

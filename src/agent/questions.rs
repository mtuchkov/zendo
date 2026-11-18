//! The questions of the agent: a form of ACP elicitation as questions with choices, and the picks of
//! the user as the content of the form (docs/zendo/activity.md: ACT-4).
use super::*;
use agent_client_protocol::v1::schema::{
    CreateElicitationRequest, CreateElicitationResponse, ElicitationAcceptAction, ElicitationAction, ElicitationContentValue, ElicitationMode,
    ElicitationPropertySchema, ElicitationSchema, ElicitationScope, EnumOption, MultiSelectItems,
};
use std::collections::BTreeMap;

/// One question of the agent: its form field, its text, its choices as value and title, one or
/// many, and the field of an own answer.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Question {
    pub field: String,
    pub text: String,
    pub choices: Vec<(String, String)>,
    pub many: bool,
    pub other: Option<String>,
}

/// The answer to one question: the values of its choices, and the own text of the user.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pick {
    pub values: Vec<String>,
    pub own: String,
}

/// The reply to a question of the agent: the picks, or `None` to cancel it.
pub(super) type Reply = Box<dyn FnOnce(Option<Vec<Pick>>) + Send>;

fn titled(o: &[EnumOption]) -> Vec<(String, String)> {
    o.iter().map(|e| (e.value.clone(), e.title.clone())).collect()
}

fn plain(v: &[String]) -> Vec<(String, String)> {
    v.iter().map(|x| (x.clone(), x.clone())).collect()
}

/// A field of a form as zendo asks it.
struct Field {
    text: Option<String>,
    choices: Vec<(String, String)>,
    many: bool,
}

/// The text, the choices, and one or many of a field; `None` for a field that zendo cannot ask.
fn field(p: &ElicitationPropertySchema) -> Option<Field> {
    match p {
        ElicitationPropertySchema::String(s) => {
            let choices = s.one_of.as_deref().map(titled).or_else(|| s.enum_values.as_deref().map(plain)).unwrap_or_default();
            Some(Field { text: s.description.clone().or_else(|| s.title.clone()), choices, many: false })
        }
        ElicitationPropertySchema::Array(a) => {
            let choices = match &a.items {
                MultiSelectItems::Titled(t) => titled(&t.options),
                MultiSelectItems::String(s) => plain(&s.values),
                _ => Vec::new(),
            };
            Some(Field { text: a.description.clone().or_else(|| a.title.clone()), choices, many: true })
        }
        _ => None,
    }
}

/// The questions of a form. A free field right after a question with choices is its own answer.
/// One question takes its text from the message.
pub(super) fn questions_of(message: &str, schema: &ElicitationSchema) -> Vec<Question> {
    let mut out: Vec<Question> = Vec::new();
    for (name, p) in &schema.properties {
        let Some(Field { text, choices, many }) = field(p) else { continue };
        if let Some(q) = out.last_mut().filter(|q| choices.is_empty() && !q.choices.is_empty() && q.other.is_none() && name.starts_with(&q.field)) {
            q.other = Some(name.clone());
            continue;
        }
        out.push(Question { field: name.clone(), text: text.unwrap_or_default(), choices, many, other: None });
    }
    if let [q] = out.as_mut_slice() {
        q.text = message.to_string();
    }
    out
}

/// The content of the form for the picks of the user.
pub(super) fn content(qs: &[Question], picks: &[Pick]) -> BTreeMap<String, ElicitationContentValue> {
    let mut m = BTreeMap::new();
    for (q, p) in qs.iter().zip(picks) {
        match (q.many, p.values.first()) {
            (true, Some(_)) => drop(m.insert(q.field.clone(), ElicitationContentValue::StringArray(p.values.clone()))),
            (false, Some(v)) => drop(m.insert(q.field.clone(), ElicitationContentValue::String(v.clone()))),
            _ => {}
        }
        if !p.own.is_empty() {
            m.insert(q.other.clone().unwrap_or_else(|| q.field.clone()), ElicitationContentValue::String(p.own.clone()));
        }
    }
    m
}

/// A form of the agent: zendo shows its questions in the chat and keeps the reply for the answer
/// of the user. A form that zendo cannot ask is declined.
pub(super) fn ask(s: &Shared, req: CreateElicitationRequest, respond: impl FnOnce(CreateElicitationResponse) + Send + 'static) {
    let ElicitationMode::Form(f) = &req.mode else { return respond(CreateElicitationResponse::new(ElicitationAction::Decline)) };
    let ElicitationScope::Session(scope) = &f.scope else { return respond(CreateElicitationResponse::new(ElicitationAction::Decline)) };
    let questions = questions_of(&req.message, &f.requested_schema);
    if questions.is_empty() {
        return respond(CreateElicitationResponse::new(ElicitationAction::Decline));
    }
    let key = s.key_of(&scope.session_id);
    let request = s.next.fetch_add(1, Ordering::Relaxed);
    let qs = questions.clone();
    let reply: Reply = Box::new(move |picks| {
        let action = match picks {
            Some(p) => ElicitationAction::Accept(ElicitationAcceptAction::new().content(content(&qs, &p))),
            None => ElicitationAction::Cancel,
        };
        respond(CreateElicitationResponse::new(action));
    });
    s.replies.lock().unwrap().insert(request, reply);
    s.emit(Event::Question { key, request, questions });
}

#[cfg(test)]
#[path = "questions_tests.rs"]
mod tests;

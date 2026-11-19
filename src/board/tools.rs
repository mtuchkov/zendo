//! The tools of the board in the MCP server `zendo` (BOARD-1, BOARD-4 to BOARD-12, BOARD-26).

use super::{Event, Journal, Kind, Msg, State, Status, inbox, inbox_of, new_id, owner_of};
use crate::app::SESSIONS;
use serde_json::{Value, json};
use std::time::Duration;

/// Where a tool runs: the journal, the repository and the session of the server.
pub struct Ctx {
    pub journal: Journal,
    pub repo: String,
    pub session: usize,
    /// The path of zendo, for the command of the watch (BOARD-25).
    pub exe: String,
    /// How long `assign` without a target waits for the choice of the user (BOARD-10).
    pub choose_for: Duration,
}

pub type Out = Result<Value, String>;

const NOT_FOUND: &str = "not found";

/// The result of the board tool `name`, or None for a tool that is not of the board.
pub fn call(ctx: &Ctx, name: &str, a: &Value) -> Option<Out> {
    let s = |k: &str| a[k].as_str().filter(|v| !v.trim().is_empty());
    Some(match name {
        "sessions" => Ok(super::tools_posts::sessions(ctx)),
        "ask" => initiate(ctx, Kind::Question, s("to"), s("text"), s("reply_to")),
        "assign" => assign(ctx, s("to"), s("text"), s("reply_to")),
        "answer" => respond(ctx, s("in_reply_to"), None, s("text")),
        "status" => respond(ctx, s("in_reply_to"), Some(s("status").and_then(Status::parse).ok_or("status is one of accepted, rejected, done, failed")), s("text")),
        "invite" => invite(ctx, s("channel"), a["session"].as_u64().map(|n| n as usize)),
        "leave" => leave(ctx, s("channel")),
        other => return super::tools_posts::call(ctx, other, a),
    })
}

/// Run `decide` under the lock of the journal.
pub(super) fn write(ctx: &Ctx, decide: impl FnOnce(&State) -> Result<Vec<Event>, String>) -> Result<(), String> {
    match ctx.journal.transact(decide) {
        Ok(r) => r.map(drop),
        Err(e) => Err(format!("the board is not available: {e}")),
    }
}

/// The result of an initiation: its id and the command that waits for its response (BOARD-26).
fn sent(ctx: &Ctx, id: &str) -> Value {
    json!({ "id": id, "wait": format!("{} board watch --session {} --for {id}", ctx.exe, ctx.session) })
}

fn initiate(ctx: &Ctx, kind: Kind, to: Option<&str>, text: Option<&str>, reply_to: Option<&str>) -> Out {
    let text = text.ok_or("text is empty")?.to_string();
    let id = new_id("m");
    write(ctx, |st| {
        let to = match to {
            Some(t) => Some(target(ctx, st, kind, t)?),
            None if kind == Kind::Task => None,
            None => return Err("`to` is empty".into()),
        };
        let reply_to = reply_to.map(|r| st.resolve_or_create(&ctx.repo, ctx.session, r).ok_or(NOT_FOUND)).transpose()?;
        Ok(vec![Event::Msg(Msg { id: id.clone(), repo: ctx.repo.clone(), from: ctx.session, to, kind, status: None, in_reply_to: None, reply_to, text })])
    })?;
    Ok(sent(ctx, &id))
}

/// The full channel of the target `t` of an initiation.
fn target(ctx: &Ctx, st: &State, kind: Kind, t: &str) -> Result<String, String> {
    let ch = st.resolve_or_create(&ctx.repo, ctx.session, t).ok_or(NOT_FOUND)?;
    match inbox_of(&ch) {
        Some(n) if n == ctx.session => Err("a session cannot send to itself".into()),
        None if kind == Kind::Task => Err("a task goes to one session: use `session:<id>` (BOARD-36)".into()),
        _ => Ok(ch),
    }
}

/// `assign`: a task; without a target the user chooses the session, and the tool waits for the
/// choice (BOARD-10).
fn assign(ctx: &Ctx, to: Option<&str>, text: Option<&str>, reply_to: Option<&str>) -> Out {
    let out = initiate(ctx, Kind::Task, to, text, reply_to)?;
    if to.is_some() {
        return Ok(out);
    }
    let id = out["id"].as_str().unwrap_or_default().to_string();
    let end = std::time::Instant::now() + ctx.choose_for;
    while std::time::Instant::now() < end {
        match ctx.journal.state().choice(&id) {
            Some(Some(n)) => return Ok(json!({ "id": id, "to": inbox(n), "wait": out["wait"] })),
            Some(None) => return Err("cancelled: the user did not choose a session".into()),
            None => std::thread::sleep(Duration::from_millis(200)),
        }
    }
    Err("cancelled: the user did not choose a session in time".into())
}

/// `answer` and `status`: a response to an initiation that this session took (BOARD-5).
fn respond(ctx: &Ctx, to: Option<&str>, status: Option<Result<Status, &str>>, text: Option<&str>) -> Out {
    let to = to.ok_or("in_reply_to is empty")?.to_string();
    let status = status.transpose()?;
    let id = new_id("m");
    write(ctx, |st| {
        let m = st.msg(&to).filter(|m| m.repo == ctx.repo && st.claimed(&m.id, ctx.session).is_some()).ok_or(NOT_FOUND)?;
        let kind = check_response(st, m, ctx.session, status)?;
        let text = text.unwrap_or(status.map_or("", Status::name)).to_string();
        if text.is_empty() {
            return Err("text is empty".into());
        }
        Ok(vec![Event::Msg(Msg { id: id.clone(), repo: ctx.repo.clone(), from: ctx.session, to: Some(m.responses_to()), kind, status, in_reply_to: Some(to.clone()), reply_to: None, text })])
    })?;
    Ok(json!({ "id": id }))
}

/// The kind of a response of session `s` to `m`: one answer to a question; to a task `accepted`
/// or `rejected`, then `done` or `failed` (BOARD-5).
fn check_response(st: &State, m: &Msg, s: usize, status: Option<Status>) -> Result<Kind, String> {
    let before: Vec<Option<Status>> = st.responses(m).filter(|r| r.from == s).map(|r| r.status).collect();
    match (m.kind, status) {
        (Kind::Question, None) if before.is_empty() => Ok(Kind::Answer),
        (Kind::Question, None) => Err("this question has your answer already".into()),
        (Kind::Task, Some(Status::Accepted | Status::Rejected)) if before.is_empty() => Ok(Kind::Status),
        (Kind::Task, Some(Status::Done | Status::Failed)) if before == [Some(Status::Accepted)] => Ok(Kind::Status),
        (Kind::Task, Some(_)) => Err("a task goes accepted or rejected, then done or failed, each once".into()),
        (Kind::Task, None) => Err("a task gets a status, not an answer".into()),
        _ => Err("no response to a response (BOARD-5)".into()),
    }
}

/// `invite`: the owner adds a session to its channel (BOARD-8).
fn invite(ctx: &Ctx, channel: Option<&str>, session: Option<usize>) -> Out {
    let session = session.filter(|n| *n < SESSIONS && *n != ctx.session).ok_or("session is one of the other sessions")?;
    write(ctx, |st| {
        let ch = st.resolve(&ctx.repo, ctx.session, channel.unwrap_or_default()).filter(|c| owner_of(c) == Some(ctx.session)).ok_or(NOT_FOUND)?;
        Ok(vec![Event::Invite { repo: ctx.repo.clone(), channel: ch, session }])
    })?;
    Ok(json!({ "ok": true }))
}

/// `leave`: a member leaves a named channel.
fn leave(ctx: &Ctx, channel: Option<&str>) -> Out {
    write(ctx, |st| {
        let ch = st.resolve(&ctx.repo, ctx.session, channel.unwrap_or_default()).filter(|c| owner_of(c).is_some()).ok_or(NOT_FOUND)?;
        Ok(vec![Event::Leave { repo: ctx.repo.clone(), channel: ch, session: ctx.session }])
    })?;
    Ok(json!({ "ok": true }))
}

#[cfg(test)]
#[path = "tools_tests.rs"]
mod tests;

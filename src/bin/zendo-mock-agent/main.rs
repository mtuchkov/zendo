//! A small ACP agent for trying zendo without Claude.
//!
//! It reads the note named in the prompt, streams a short answer and
//! proposes one edit to that note through a permission request.
//! Say "shell" in the prompt to make it ask for a shell command instead.

mod tasks;
mod turn;

use tasks::*;
use turn::*;
use agent_client_protocol::v1::schema::{
    AgentCapabilities, AvailableCommand, AvailableCommandInput, AvailableCommandsUpdate, UnstructuredCommandInput, PromptCapabilities, ContentBlock, ContentChunk, Diff, Implementation, InitializeRequest,
    InitializeResponse, LoadSessionRequest, LoadSessionResponse, NewSessionRequest,
    NewSessionResponse, PermissionOption, PermissionOptionKind, PromptRequest, PromptResponse,
    RequestPermissionOutcome, RequestPermissionRequest, SessionNotification, SessionUpdate,
    StopReason, TextContent, ToolCall, ToolCallContent, ToolCallStatus, ToolCallUpdate,
    ToolCallUpdateFields, ToolKind,
};
use agent_client_protocol::{Agent, Client, Result, Stdio};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

static NEXT: AtomicU64 = AtomicU64::new(1);

type Cx = agent_client_protocol::v1::ConnectionTo<Client>;

fn say(cx: &Cx, sid: &agent_client_protocol::v1::schema::SessionId, text: &str) -> Result<()> {
    cx.send_notification(SessionNotification::new(
        sid.clone(),
        SessionUpdate::AgentMessageChunk(ContentChunk::new(ContentBlock::Text(TextContent::new(text)))),
    ))
}

async fn stream(cx: &Cx, sid: &agent_client_protocol::v1::schema::SessionId, text: &str) -> Result<()> {
    for word in text.split_inclusive(' ') {
        say(cx, sid, word)?;
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    Ok(())
}

/// The slash commands of the mock agent.
fn commands(cx: &Cx, sid: &agent_client_protocol::v1::schema::SessionId) -> Result<()> {
    let list = vec![
        AvailableCommand::new("review", "review the note").input(AvailableCommandInput::Unstructured(UnstructuredCommandInput::new("what to look at"))),
        AvailableCommand::new("compact", "make the thread short"),
    ];
    cx.send_notification(SessionNotification::new(sid.clone(), SessionUpdate::AvailableCommandsUpdate(AvailableCommandsUpdate::new(list))))
}

fn prompt_text(p: &PromptRequest) -> String {
    p.prompt
        .iter()
        .filter_map(|b| match b {
            ContentBlock::Text(t) => Some(t.text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Insert one EARS line after the last bullet, or at the end.
fn propose(old: &str) -> String {
    // For demos and screenshots: ZENDO_MOCK_LINE is the line that the mock agent proposes.
    let custom = std::env::var("ZENDO_MOCK_LINE").ok();
    let line = custom.as_deref().unwrap_or("- WHEN the mock agent proposes a change, zendo shall show it as a diff before it writes the file.");
    let mut lines: Vec<&str> = old.lines().collect();
    let at = lines.iter().rposition(|l| l.trim_start().starts_with("- ")).map(|i| i + 1).unwrap_or(lines.len());
    lines.insert(at, line);
    let mut out = lines.join("\n");
    if old.ends_with('\n') {
        out.push('\n');
    }
    out
}


#[tokio::main]
async fn main() -> Result<()> {
    Agent
        .v1()
        .name("zendo-mock-agent")
        .on_receive_request(
            async move |init: InitializeRequest, responder, _cx| {
                responder.respond(
                    InitializeResponse::new(init.protocol_version)
                        .agent_capabilities(AgentCapabilities::new().load_session(true).prompt_capabilities(PromptCapabilities::new().image(true)))
                        .agent_info(Implementation::new("zendo-mock-agent", env!("CARGO_PKG_VERSION"))),
                )
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |_req: NewSessionRequest, responder, cx: Cx| {
                let id = format!("mock-{}", std::process::id() as u64 * 1000 + NEXT.fetch_add(1, Ordering::Relaxed));
                responder.respond(NewSessionResponse::new(id.clone()))?;
                commands(&cx, &id.into())
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |req: LoadSessionRequest, responder, cx: Cx| {
                let sid = req.session_id.clone();
                cx.send_notification(SessionNotification::new(
                    sid.clone(),
                    SessionUpdate::UserMessageChunk(ContentChunk::new(ContentBlock::Text(TextContent::new("(earlier question)")))),
                ))?;
                say(&cx, &sid, "(earlier answer, replayed by the mock agent)")?;
                commands(&cx, &sid)?;
                responder.respond(LoadSessionResponse::new())
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |req: PromptRequest, responder, cx: Cx| {
                let cx2 = cx.clone();
                cx.spawn(async move {
                    match turn(cx2, req).await {
                        Ok(stop) => responder.respond(PromptResponse::new(stop)),
                        Err(e) => responder.respond_with_error(e),
                    }
                })
            },
            agent_client_protocol::on_receive_request!(),
        )
        .connect_to(Stdio::new())
        .await
}

#[cfg(test)]
#[path = "main_tests.rs"]
mod tests;

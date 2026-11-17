//! The sessions of the agent: open one for a key (a new one, or an earlier one that it loads), and
//! close one that zendo no longer uses.
use super::*;

pub(super) async fn open_session(
    cx: &agent_client_protocol::v1::ConnectionTo<Agent>,
    shared: &Arc<Shared>,
    key: &str,
    known: Option<String>,
    can_load: bool,
    cwd: &std::path::Path,
) -> Result<SessionId, agent_client_protocol::Error> {
    if let (Some(known), true) = (known, can_load) {
        let id = SessionId::new(known.clone());
        shared.keys.lock().unwrap().insert(known.clone(), key.to_string());
        *shared.loading.lock().unwrap() = Some(key.to_string());
        let r = cx.send_request(LoadSessionRequest::new(id.clone(), cwd.to_path_buf()).mcp_servers(crate::mcp::acp_servers(key))).block_task().await;
        *shared.loading.lock().unwrap() = None;
        match r {
            Ok(_) => {
                shared.emit(Event::Session { key: key.to_string(), id: known, resumed: true });
                return Ok(id);
            }
            Err(_) => shared.emit(Event::Error { key: Some(key.to_string()), text: "could not resume the earlier thread, so this is a new one (details in .zendo/agent.log)".into() }),
        }
    }
    new_session(cx, shared, key, cwd).await.map(|(id, _)| id)
}

/// A new session for `key`, and the config options of the agent for it.
pub(super) async fn new_session(
    cx: &agent_client_protocol::v1::ConnectionTo<Agent>,
    shared: &Arc<Shared>,
    key: &str,
    cwd: &std::path::Path,
) -> Result<(SessionId, Vec<SessionConfigOption>), agent_client_protocol::Error> {
    let r = cx.send_request(NewSessionRequest::new(cwd.to_path_buf()).mcp_servers(crate::mcp::acp_servers(key))).block_task().await?;
    let id = r.session_id;
    shared.keys.lock().unwrap().insert(id.0.to_string(), key.to_string());
    shared.emit(Event::Session { key: key.to_string(), id: id.0.to_string(), resumed: false });
    Ok((id, r.config_options.unwrap_or_default()))
}

/// A new session for `key` with the model of the agent that `model` names, if the agent has one
/// (NAMES-12).
pub(super) async fn model_session(
    cx: &agent_client_protocol::v1::ConnectionTo<Agent>,
    shared: &Arc<Shared>,
    key: &str,
    model: &str,
    cwd: &std::path::Path,
) -> Result<SessionId, agent_client_protocol::Error> {
    let (id, options) = new_session(cx, shared, key, cwd).await?;
    if let Some((config, value)) = super::model::choose(&options, model) {
        // A model that the agent refuses leaves its default model (NAMES-13).
        let _ = cx.send_request(SetSessionConfigOptionRequest::new(id.clone(), config, value.as_str())).block_task().await;
    }
    Ok(id)
}

/// Close a session that zendo no longer uses, where the agent can, so that the agent frees what it
/// holds for it: its process and its MCP servers (issue #5). A pending turn of it ends as cancelled.
/// It waits for the answer: a next command can load the same session again (`cli_tick`).
pub(super) async fn close_session(cx: &agent_client_protocol::v1::ConnectionTo<Agent>, id: SessionId, can_close: bool) {
    if can_close {
        // An error leaves the session open, as before.
        let _ = cx.send_request(CloseSessionRequest::new(id)).block_task().await;
    }
}

//! The ACP connection: one background task that talks to the agent process.
use super::*;

pub(super) struct Shared {
    pub(super) events: UnboundedSender<Event>,
    /// Session id → note key.
    pub(super) keys: Mutex<HashMap<String, String>>,
    pub(super) answers: Mutex<HashMap<u64, Answer>>,
    /// The open questions of the agent and their replies (ACT-4).
    pub(super) replies: Mutex<HashMap<u64, super::questions::Reply>>,
    pub(super) next: AtomicU64,
    /// While a session is loading, its replayed messages carry this key.
    pub(super) loading: Mutex<Option<String>>,
}

impl Shared {
    pub(super) fn key_of(&self, id: &SessionId) -> String {
        let id = id.0.to_string();
        if let Some(k) = self.keys.lock().unwrap().get(&id) {
            return k.clone();
        }
        self.loading.lock().unwrap().clone().unwrap_or(id)
    }
    pub(super) fn emit(&self, e: Event) {
        let _ = self.events.send(e);
    }
}

pub(super) fn kind_name(k: &ToolKind) -> String {
    format!("{k:?}").to_lowercase()
}

pub(super) async fn run(
    command: String,
    env: BTreeMap<String, String>,
    cwd: PathBuf,
    log: Option<PathBuf>,
    cmds: UnboundedReceiver<Cmd>,
    shared: Arc<Shared>,
) -> Result<(), agent_client_protocol::Error> {
    let mut agent = AcpAgent::new(AcpAgent::from_str(&command)?.into_config().envs(env));
    if let Some(path) = log {
        let file = Arc::new(Mutex::new(std::fs::OpenOptions::new().create(true).append(true).open(path).ok()));
        agent = agent.with_debug(move |line, dir| {
            if let Some(f) = file.lock().unwrap().as_mut() {
                // The local time of each line dates a stuck turn.
                let _ = writeln!(f, "{} {dir:?} {line}", chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f"));
            }
        });
    }
    let on_note = shared.clone();
    let on_ask = shared.clone();
    let on_form = shared.clone();
    Client
        .v1()
        .name("zendo")
        .on_receive_notification(
            async move |n: SessionNotification, _cx| {
                on_notification(&on_note, n);
                Ok(())
            },
            agent_client_protocol::on_receive_notification!(),
        )
        .on_receive_request(
            async move |req: RequestPermissionRequest, responder, _cx| {
                let s = &on_ask;
                let key = s.key_of(&req.session_id);
                let f = &req.tool_call.fields;
                let title = f.title.clone().unwrap_or_else(|| "use a tool".into());
                // Policy: zendo never lets the agent run shell commands.
                if matches!(f.kind, Some(ToolKind::Execute)) {
                    s.emit(Event::Blocked { key, title });
                    return responder.respond(RequestPermissionResponse::new(outcome(reject_option(&req))));
                }
                let kind = f.kind.as_ref().map(kind_name).unwrap_or_default();
                let (diffs, choices) = (diffs_of(&req), choices_of(&req));
                let request = s.next.fetch_add(1, Ordering::Relaxed);
                let answer: Answer = Box::new(move |option: Option<String>| {
                    let _ = responder.respond(RequestPermissionResponse::new(outcome(option.map(Into::into))));
                });
                s.answers.lock().unwrap().insert(request, answer);
                s.emit(Event::Permission { key, request, title, kind, diffs, choices });
                Ok(())
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |req: agent_client_protocol::v1::schema::CreateElicitationRequest, responder, _cx| {
                super::questions::ask(&on_form, req, move |r| drop(responder.respond(r)));
                Ok(())
            },
            agent_client_protocol::on_receive_request!(),
        )
        .connect_with(agent, async move |cx: agent_client_protocol::v1::ConnectionTo<Agent>| serve(cx, shared, cmds, cwd).await)
        .await
}

/// A session update: the text, thoughts and tool calls of a turn, or the replay of a loaded thread.
fn on_notification(s: &Shared, n: SessionNotification) {
    let key = s.key_of(&n.session_id);
    let replaying = s.loading.lock().unwrap().is_some();
    if let SessionUpdate::AvailableCommandsUpdate(u) = n.update {
        return s.emit(Event::Commands { list: commands_of(u) });
    }
    if replaying {
        let (user, block) = match n.update {
            SessionUpdate::UserMessageChunk(c) => (true, c.content),
            SessionUpdate::AgentMessageChunk(c) => (false, c.content),
            _ => return,
        };
        if let ContentBlock::Text(t) = block {
            s.emit(Event::Replay { key, user, text: t.text });
        }
        return;
    }
    match n.update {
        SessionUpdate::AgentMessageChunk(c) => {
            if let ContentBlock::Text(t) = c.content {
                s.emit(Event::Text { key, text: t.text });
            }
        }
        SessionUpdate::UserMessageChunk(c) => {
            if let ContentBlock::Text(t) = c.content {
                s.emit(Event::UserText { key, text: t.text });
            }
        }
        SessionUpdate::AgentThoughtChunk(_) => s.emit(Event::Thought { key }),
        SessionUpdate::ToolCall(tc) => s.emit(super::tool_events::call(key, tc)),
        SessionUpdate::ToolCallUpdate(u) => s.emit(super::tool_events::update(key, u)),
        _ => {}
    }
}

fn commands_of(u: AvailableCommandsUpdate) -> Vec<Command> {
    u.available_commands
        .into_iter()
        .map(|c| Command {
            hint: c.input.map(|i| match i {
                AvailableCommandInput::Unstructured(x) => x.hint,
                _ => String::new(),
            }),
            name: c.name,
            description: c.description,
        })
        .collect()
}

fn outcome(option: Option<agent_client_protocol::v1::schema::PermissionOptionId>) -> RequestPermissionOutcome {
    match option {
        Some(id) => RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(id)),
        None => RequestPermissionOutcome::Cancelled,
    }
}

/// The option that rejects the request, once or always.
fn reject_option(req: &RequestPermissionRequest) -> Option<agent_client_protocol::v1::schema::PermissionOptionId> {
    [PermissionOptionKind::RejectOnce, PermissionOptionKind::RejectAlways].iter().find_map(|k| req.options.iter().find(|o| o.kind == *k).map(|o| o.option_id.clone()))
}

fn diffs_of(req: &RequestPermissionRequest) -> Vec<DiffView> {
    let f = &req.tool_call.fields;
    f.content
        .iter()
        .flatten()
        .filter_map(|c| match c {
            ToolCallContent::Diff(d) => Some(DiffView { path: d.path.clone(), old: d.old_text.clone(), new: d.new_text.clone() }),
            _ => None,
        })
        .collect()
}

fn choices_of(req: &RequestPermissionRequest) -> Vec<Choice> {
    req.options
        .iter()
        .map(|o| Choice {
            id: o.option_id.0.to_string(),
            name: o.name.clone(),
            allow: matches!(o.kind, PermissionOptionKind::AllowOnce | PermissionOptionKind::AllowAlways),
            always: matches!(o.kind, PermissionOptionKind::AllowAlways | PermissionOptionKind::RejectAlways),
        })
        .collect()
}

/// What zendo tells the agent that it can show: questions with choices, and the output and the exit
/// code of a command (ADR 0014, ACT-4, ACT-6).
fn capabilities() -> agent_client_protocol::v1::schema::ClientCapabilities {
    let mut meta = serde_json::Map::new();
    meta.insert("terminal_output".into(), serde_json::Value::Bool(true));
    let form = agent_client_protocol::v1::schema::ElicitationCapabilities::new().form(agent_client_protocol::v1::schema::ElicitationFormCapabilities::new());
    agent_client_protocol::v1::schema::ClientCapabilities::new().elicitation(form).meta(meta)
}

/// The connection: the handshake, then the commands of zendo until it closes.
/// A turn runs beside the commands of zendo; its end, or its error, goes to the app.
fn start_turn(cx: &agent_client_protocol::v1::ConnectionTo<Agent>, shared: &Arc<Shared>, key: String, req: PromptRequest) -> Result<(), agent_client_protocol::Error> {
    let (cx2, s) = (cx.clone(), shared.clone());
    cx.spawn(async move {
        match cx2.send_request(req).block_task().await {
            Ok(r) => s.emit(Event::TurnEnd { key, stop: format!("{:?}", r.stop_reason).to_lowercase() }),
            Err(e) => {
                s.emit(Event::Error { key: Some(key.clone()), text: e.to_string() });
                s.emit(Event::TurnEnd { key, stop: "error".into() });
            }
        }
        Ok(())
    })
}

async fn serve(cx: agent_client_protocol::v1::ConnectionTo<Agent>, shared: Arc<Shared>, mut cmds: UnboundedReceiver<Cmd>, cwd: PathBuf) -> Result<(), agent_client_protocol::Error> {
    let init = cx.send_request(InitializeRequest::new(ProtocolVersion::V1).client_capabilities(capabilities())).block_task().await?;
    let can_load = init.agent_capabilities.load_session;
    let can_image = init.agent_capabilities.prompt_capabilities.image;
    let can_close = init.agent_capabilities.session_capabilities.close.is_some();
    let name = init.agent_info.as_ref().map(|i| i.name.clone()).unwrap_or_else(|| "agent".into());
    shared.emit(Event::Ready { agent: name, can_load, can_image });

    let mut sessions: HashMap<String, SessionId> = HashMap::new();
    loop {
        let cmd = tokio::select! {
            c = cmds.recv() => match c { Some(c) => c, None => return Ok(()) },
            _ = cx.terminated() => return Ok(()),
        };
        match cmd {
            Cmd::Prompt { key, known, text, attach } => {
                let id = match sessions.get(&key) {
                    Some(id) => id.clone(),
                    None => {
                        let id = open_session(&cx, &shared, &key, known, can_load, &cwd).await?;
                        sessions.insert(key.clone(), id.clone());
                        id
                    }
                };
                start_turn(&cx, &shared, key, PromptRequest::new(id, blocks(text, &attach, can_image)))?;
            }
            Cmd::Open { key, known } => {
                if let std::collections::hash_map::Entry::Vacant(e) = sessions.entry(key) {
                    let id = open_session(&cx, &shared, e.key(), known, can_load, &cwd).await?;
                    e.insert(id);
                }
            }
            Cmd::Answer { request, option } => {
                if let Some(answer) = shared.answers.lock().unwrap().remove(&request) {
                    answer(option);
                }
            }
            Cmd::Reply { request, picks } => {
                if let Some(reply) = shared.replies.lock().unwrap().remove(&request) {
                    reply(picks);
                }
            }
            Cmd::Cancel { key } => {
                if let Some(id) = sessions.get(&key) {
                    cx.send_notification(CancelNotification::new(id.clone()))?;
                }
            }
            Cmd::Forget { key } => {
                if let Some(old) = sessions.remove(&key) {
                    close_session(&cx, old, can_close).await;
                }
            }
            Cmd::Model { key, model } => {
                let id = model_session(&cx, &shared, &key, &model, &cwd).await?;
                if let Some(old) = sessions.insert(key, id) {
                    close_session(&cx, old, can_close).await;
                }
            }
        }
    }
}

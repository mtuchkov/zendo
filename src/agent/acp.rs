//! The ACP connection: one background task that talks to the agent process.
use super::*;

pub(super) struct Shared {
    pub(super) events: UnboundedSender<Event>,
    /// Session id → note key.
    pub(super) keys: Mutex<HashMap<String, String>>,
    pub(super) answers: Mutex<HashMap<u64, Answer>>,
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
                let _ = writeln!(f, "{dir:?} {line}");
            }
        });
    }
    let on_note = shared.clone();
    let on_ask = shared.clone();
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
        .connect_with(agent, async move |cx: agent_client_protocol::v1::ConnectionTo<Agent>| serve(cx, shared, cmds, cwd).await)
        .await
}

/// A session update: the text, thoughts and tool calls of a turn, or the replay of a loaded thread.
fn on_notification(s: &Shared, n: SessionNotification) {
    let key = s.key_of(&n.session_id);
    let replaying = s.loading.lock().unwrap().is_some();
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
        SessionUpdate::ToolCall(tc) => s.emit(Event::Tool {
            key,
            id: tc.tool_call_id.0.to_string(),
            title: tc.title,
            kind: kind_name(&tc.kind),
            done: format!("{:?}", tc.status) == "Completed",
        }),
        SessionUpdate::ToolCallUpdate(u) => s.emit(Event::Tool {
            key,
            id: u.tool_call_id.0.to_string(),
            title: u.fields.title.unwrap_or_default(),
            kind: u.fields.kind.as_ref().map(kind_name).unwrap_or_default(),
            done: u.fields.status.map(|st| format!("{st:?}") == "Completed").unwrap_or(false),
        }),
        _ => {}
    }
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

/// The connection: the handshake, then the commands of zendo until it closes.
async fn serve(cx: agent_client_protocol::v1::ConnectionTo<Agent>, shared: Arc<Shared>, mut cmds: UnboundedReceiver<Cmd>, cwd: PathBuf) -> Result<(), agent_client_protocol::Error> {
    let init = cx.send_request(InitializeRequest::new(ProtocolVersion::V1)).block_task().await?;
    let can_load = init.agent_capabilities.load_session;
    let can_image = init.agent_capabilities.prompt_capabilities.image;
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
                let req = PromptRequest::new(id, blocks(text, &attach, can_image));
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
                })?;
            }
            Cmd::Answer { request, option } => {
                if let Some(answer) = shared.answers.lock().unwrap().remove(&request) {
                    answer(option);
                }
            }
            Cmd::Cancel { key } => {
                if let Some(id) = sessions.get(&key) {
                    cx.send_notification(CancelNotification::new(id.clone()))?;
                }
            }
            Cmd::Forget { key } => {
                sessions.remove(&key);
            }
        }
    }
}

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
        let r = cx.send_request(LoadSessionRequest::new(id.clone(), cwd.to_path_buf())).block_task().await;
        *shared.loading.lock().unwrap() = None;
        match r {
            Ok(_) => {
                shared.emit(Event::Session { key: key.to_string(), id: known, resumed: true });
                return Ok(id);
            }
            Err(_) => shared.emit(Event::Error { key: Some(key.to_string()), text: "could not resume the earlier thread, so this is a new one (details in .zendo/agent.log)".into() }),
        }
    }
    let r = cx.send_request(NewSessionRequest::new(cwd.to_path_buf())).block_task().await?;
    let id = r.session_id;
    shared.keys.lock().unwrap().insert(id.0.to_string(), key.to_string());
    shared.emit(Event::Session { key: key.to_string(), id: id.0.to_string(), resumed: false });
    Ok(id)
}

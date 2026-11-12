//! One turn of the mock agent: the scripted tasks, a blocked shell command, or an answer that
//! proposes one line for the note.
use super::*;

pub(super) async fn turn(cx: Cx, p: PromptRequest) -> Result<StopReason> {
    let sid = p.session_id.clone();
    let text = prompt_text(&p);
    let file = text
        .lines()
        .find_map(|l| l.trim().strip_prefix("Note file: "))
        .map(|s| PathBuf::from(s.trim()));
    let question = text.rsplit("Question:").next().unwrap_or("").trim().to_string();
    if let Some(stop) = scripted(&cx, &sid, &text).await? {
        return Ok(stop);
    }
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    if let Some(path) = &file {
        let id = format!("read-{n}");
        cx.send_notification(SessionNotification::new(
            sid.clone(),
            SessionUpdate::ToolCall(ToolCall::new(id, format!("Read {}", path.display())).kind(ToolKind::Read).status(ToolCallStatus::Completed)),
        ))?;
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
    if question.to_lowercase().contains("shell") {
        return ask_shell(&cx, &sid, n).await;
    }
    answer(&cx, &sid, &p, &question).await?;
    let Some(path) = file else {
        stream(&cx, &sid, "No note file was in the prompt.").await?;
        return Ok(StopReason::EndTurn);
    };
    propose_edit(&cx, &sid, &path, n).await
}

/// The tasks of zendo that need no note: fix the structure, metadata, move notes, a recap.
async fn scripted(cx: &Cx, sid: &agent_client_protocol::v1::schema::SessionId, text: &str) -> Result<Option<StopReason>> {
    if text.contains("You are fixing the project structure file") {
        return fix_structure(cx, sid, text).await.map(Some);
    }
    if text.contains("Write metadata for the markdown document") {
        let mut out = String::new();
        if text.contains("TITLE: <the title>") {
            out.push_str("TITLE: A mock title\n");
        }
        if text.contains("FRONT MATTER:") {
            out.push_str("FRONT MATTER:\nstatus: draft\ndate: 2026-11-12\nlevel: what the mock agent can see\n");
        }
        out.push_str("END");
        stream(cx, sid, &out).await?;
        return Ok(Some(StopReason::EndTurn));
    }
    if text.contains("You are moving the notes") || text.contains("You are consolidating the notes") {
        return move_notes(cx, sid, text).await.map(Some);
    }
    if text.contains("Write a recap") {
        let title = text.lines().find_map(|l| l.strip_prefix("# ")).unwrap_or("this note").trim().to_string();
        let changed = if text.contains("--- changes since the last recap") { "Changed since the last recap: one new requirement." } else { "First recap of this note." };
        stream(cx, sid, &format!("- Decides: {title}.\n- {changed}\n- Open: what is the retention for the log?")).await?;
        return Ok(Some(StopReason::EndTurn));
    }
    Ok(None)
}

/// A question about the shell: the mock asks to run a command, and zendo must reject it.
async fn ask_shell(cx: &Cx, sid: &agent_client_protocol::v1::schema::SessionId, n: u64) -> Result<StopReason> {
    stream(cx, sid, "I want to check the git log first. ").await?;
    let call = ToolCallUpdate::new(format!("bash-{n}"), ToolCallUpdateFields::new().title("git log --oneline -5".to_string()).kind(ToolKind::Execute));
    let options = vec![
        PermissionOption::new("allow", "Allow", PermissionOptionKind::AllowOnce),
        PermissionOption::new("reject", "Reject", PermissionOptionKind::RejectOnce),
    ];
    let r = cx.send_request(RequestPermissionRequest::new(sid.clone(), call, options)).block_task().await?;
    let allowed = matches!(&r.outcome, RequestPermissionOutcome::Selected(s) if s.option_id.0.as_ref() == "allow");
    stream(cx, sid, if allowed { "The command ran." } else { "The command was blocked, so I will answer from the note only." }).await?;
    Ok(StopReason::EndTurn)
}

/// The text of the answer: what came attached, then the scripted reply or the mock text.
async fn answer(cx: &Cx, sid: &agent_client_protocol::v1::schema::SessionId, p: &PromptRequest, question: &str) -> Result<()> {
    let attached: Vec<String> = p
        .prompt
        .iter()
        .filter_map(|b| match b {
            ContentBlock::Image(i) => Some(format!("an image ({}, {} bytes of base64)", i.mime_type, i.data.len())),
            ContentBlock::ResourceLink(r) => Some(format!("a file link ({})", r.uri)),
            _ => None,
        })
        .collect();
    if !attached.is_empty() {
        stream(cx, sid, &format!("I got {}. ", attached.join(", "))).await?;
    }
    // For demos: ZENDO_MOCK_REPLY is the scripted answer, in place of the mock text.
    match std::env::var("ZENDO_MOCK_REPLY") {
        Ok(reply) => {
            for chunk in reply.split_inclusive(' ') {
                stream(cx, sid, chunk).await?;
                tokio::time::sleep(std::time::Duration::from_millis(25)).await;
            }
        }
        Err(_) => stream(cx, sid, &format!("You asked: \"{question}\". I am the mock agent, so I cannot think, but I can show the flow. ")).await?,
    }
    Ok(())
}

/// Propose one line as an edit, and write it only when zendo allows it.
async fn propose_edit(cx: &Cx, sid: &agent_client_protocol::v1::schema::SessionId, path: &std::path::Path, n: u64) -> Result<StopReason> {
    let old = std::fs::read_to_string(path).unwrap_or_default();
    let new = propose(&old);
    if std::env::var("ZENDO_MOCK_REPLY").is_err() {
        stream(cx, sid, "I propose one line for the note:").await?;
    }
    let id = format!("edit-{n}");
    let diff = ToolCallContent::Diff(Diff::new(path.to_path_buf(), new.clone()).old_text(old.clone()));
    let fields = ToolCallUpdateFields::new()
        .title(format!("Edit {}", path.file_name().unwrap_or_default().to_string_lossy()))
        .kind(ToolKind::Edit)
        .status(ToolCallStatus::Pending)
        .content(vec![diff]);
    let options = vec![
        PermissionOption::new("allow", "Allow", PermissionOptionKind::AllowOnce),
        PermissionOption::new("reject", "Reject", PermissionOptionKind::RejectOnce),
    ];
    let r = cx
        .send_request(RequestPermissionRequest::new(sid.clone(), ToolCallUpdate::new(id.clone(), fields), options))
        .block_task()
        .await?;
    let allowed = matches!(&r.outcome, RequestPermissionOutcome::Selected(s) if s.option_id.0.as_ref() == "allow");
    if allowed {
        std::fs::write(path, &new).map_err(agent_client_protocol::Error::into_internal_error)?;
        cx.send_notification(SessionNotification::new(
            sid.clone(),
            SessionUpdate::ToolCallUpdate(ToolCallUpdate::new(id, ToolCallUpdateFields::new().status(ToolCallStatus::Completed))),
        ))?;
        stream(cx, sid, "\n\nDone. The line is in the note now.").await?;
    } else {
        stream(cx, sid, "\n\nOK, I left the note as it is.").await?;
    }
    Ok(StopReason::EndTurn)
}

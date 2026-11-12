//! Scripted answers to the tasks of zendo: an edit, a fixed structure file, moved notes.
use super::*;

/// Ask to edit a file; true when the client allows it.
pub(super) async fn ask_edit(cx: &Cx, sid: &agent_client_protocol::v1::schema::SessionId, path: &std::path::Path, old: Option<String>, new: &str) -> Result<bool> {
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let mut diff = Diff::new(path.to_path_buf(), new.to_string());
    if let Some(o) = old {
        diff = diff.old_text(o);
    }
    let fields = ToolCallUpdateFields::new()
        .title(format!("Edit {}", path.file_name().unwrap_or_default().to_string_lossy()))
        .kind(ToolKind::Edit)
        .content(vec![ToolCallContent::Diff(diff)]);
    let options = vec![
        PermissionOption::new("allow", "Allow", PermissionOptionKind::AllowOnce),
        PermissionOption::new("reject", "Reject", PermissionOptionKind::RejectOnce),
    ];
    let r = cx.send_request(RequestPermissionRequest::new(sid.clone(), ToolCallUpdate::new(format!("edit-{n}"), fields), options)).block_task().await?;
    Ok(matches!(&r.outcome, RequestPermissionOutcome::Selected(s) if s.option_id.0.as_ref() == "allow"))
}

/// `zendo doctor --fix`: first try another file (zendo must refuse), then write a valid structure file.
pub(super) async fn fix_structure(cx: &Cx, sid: &agent_client_protocol::v1::schema::SessionId, text: &str) -> Result<StopReason> {
    let Some(file) = text.lines().find_map(|l| l.strip_prefix("Task: make ")).and_then(|r| r.split(" a valid").next()).map(PathBuf::from) else {
        stream(cx, sid, "I did not find the file name.").await?;
        return Ok(StopReason::EndTurn);
    };
    let other = file.with_file_name("README.md");
    let other_ok = ask_edit(cx, sid, &other, std::fs::read_to_string(&other).ok(), "# changed by the mock agent\n").await?;
    let new = "# Fixed by the mock agent.\n[project]\nname = \"mock\"\n\n[[entry]]\npath = \"docs\"\nabout = \"The documentation.\"\nmain = true\n";
    let ok = ask_edit(cx, sid, &file, std::fs::read_to_string(&file).ok(), new).await?;
    if ok {
        std::fs::write(&file, new).map_err(agent_client_protocol::Error::into_internal_error)?;
    }
    stream(cx, sid, &format!("Rewrote the file with one main entry.\nThe edit of README.md was {}.", if other_ok { "allowed" } else { "refused" })).await?;
    Ok(StopReason::EndTurn)
}

/// `zendo notes move` and `consolidate`: first try a file outside the target (zendo must refuse), then
/// write each listed note into the target folder (append when the name is there) and empty the old file.
pub(super) async fn move_notes(cx: &Cx, sid: &agent_client_protocol::v1::schema::SessionId, text: &str) -> Result<StopReason> {
    let Some(to) = text.lines().find_map(|l| l.strip_prefix("Target folder: ")).map(|t| PathBuf::from(t.trim())) else {
        stream(cx, sid, "I did not find the target folder.").await?;
        return Ok(StopReason::EndTurn);
    };
    let outside = to.parent().unwrap_or(&to).join("outside.md");
    let outside_ok = ask_edit(cx, sid, &outside, None, "# not allowed\n").await?;
    let files: Vec<PathBuf> = text.lines().filter_map(|l| l.trim().strip_prefix("- ")).map(PathBuf::from).filter(|p| p.is_absolute() && p.is_file()).collect();
    let mut moved = 0;
    for f in &files {
        let src = std::fs::read_to_string(f).unwrap_or_default();
        let dest = to.join(f.file_name().unwrap_or_default());
        let old = std::fs::read_to_string(&dest).ok();
        let new = match &old {
            Some(o) => format!("{}\n\n---\n\n{src}", o.trim_end()),
            None => src.clone(),
        };
        if ask_edit(cx, sid, &dest, old, &new).await? {
            std::fs::write(&dest, &new).map_err(agent_client_protocol::Error::into_internal_error)?;
            if ask_edit(cx, sid, f, Some(src), "").await? {
                std::fs::write(f, "").map_err(agent_client_protocol::Error::into_internal_error)?;
                moved += 1;
            }
        }
    }
    stream(cx, sid, &format!("Moved {moved} notes.\nThe edit outside the target was {}.", if outside_ok { "allowed" } else { "refused" })).await?;
    Ok(StopReason::EndTurn)
}

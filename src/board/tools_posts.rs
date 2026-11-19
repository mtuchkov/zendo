//! The tools `sessions`, `post`, `release`, `posts` and `tombstone` (BOARD-10, BOARD-13 to
//! BOARD-19, BOARD-29 to BOARD-34).

use super::tools::{Ctx, Out, write};
use super::{Event, Post, PostState, State, new_id, now_ms, proc_start};
use crate::app::SESSIONS;
use serde_json::{Value, json};

pub(super) fn call(ctx: &Ctx, name: &str, a: &Value) -> Option<Out> {
    let s = |k: &str| a[k].as_str().filter(|v| !v.trim().is_empty());
    Some(match name {
        "post" => post(ctx, s("resource"), s("text"), a["pid"].as_u64().map(|p| p as u32)),
        "release" => release(ctx, s("resource")),
        "posts" => Ok(posts(ctx)),
        "tombstone" => tombstone(ctx, s("post")),
        _ => return None,
    })
}

/// The sessions of the repository: id, name, state and the branch of the worktree (BOARD-10).
pub(super) fn sessions(ctx: &Ctx) -> Value {
    let beats: Vec<_> = ctx.journal.beats().into_iter().filter(|b| b.repo == ctx.repo).collect();
    let fresh = beats.iter().max_by_key(|b| b.at);
    let list: Vec<Value> = (0..SESSIONS)
        .map(|n| {
            let info = fresh.and_then(|b| b.sessions.iter().find(|s| s.id == n));
            let dir = if n == 0 { ctx.repo.clone() } else { format!("{}/.worktrees/{n}", ctx.repo) };
            json!({
                "id": n,
                "inbox": super::inbox(n),
                "name": info.map(|i| i.name.clone()).unwrap_or_default(),
                "state": info.map_or("unknown", |i| i.state.as_str()),
                "branch": branch(&dir),
                "you": n == ctx.session,
            })
        })
        .collect();
    json!({ "sessions": list })
}

/// The branch checked out in `dir`, or null.
fn branch(dir: &str) -> Value {
    let out = std::process::Command::new("git").args(["-C", dir, "branch", "--show-current"]).output();
    out.ok().filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).filter(|b| !b.is_empty()).map_or(Value::Null, Value::String)
}

fn post(ctx: &Ctx, resource: Option<&str>, text: Option<&str>, pid: Option<u32>) -> Out {
    let resource = resource.ok_or("resource is empty")?.to_string();
    let text = text.ok_or("text is empty")?.to_string();
    let id = new_id("p");
    let start = pid.and_then(proc_start);
    write(ctx, |st| {
        let mut ev: Vec<Event> = st.post_of(&ctx.repo, ctx.session, &resource).map(|p| Event::Release { post: p.post.id.clone() }).into_iter().collect();
        ev.push(Event::Post(Post { id: id.clone(), repo: ctx.repo.clone(), session: ctx.session, resource, text, pid, start }));
        Ok(ev)
    })?;
    Ok(json!({ "id": id }))
}

fn release(ctx: &Ctx, resource: Option<&str>) -> Out {
    let resource = resource.ok_or("resource is empty")?;
    write(ctx, |st| {
        let p = st.post_of(&ctx.repo, ctx.session, resource).ok_or("not found")?;
        Ok(vec![Event::Release { post: p.post.id.clone() }])
    })?;
    Ok(json!({ "ok": true }))
}

/// One post as JSON, with its lease (BOARD-19, BOARD-32).
pub fn post_json(st: &PostState, beat: Option<u64>, now: u64) -> Value {
    let lease = st.lease(now, beat);
    let mut v = json!({
        "id": st.post.id, "repo": st.post.repo, "session": st.post.session, "resource": st.post.resource,
        "text": st.post.text, "posted_at": st.at, "lease_live": lease.live, "lease_since": lease.since,
    });
    if !lease.live {
        v["expired_for_seconds"] = json!(now.saturating_sub(lease.since) / 1000);
    }
    if let Some(end) = &st.end {
        v["ended"] = json!(format!("{end:?}"));
    }
    v
}

fn posts(ctx: &Ctx) -> Value {
    let st = ctx.journal.state();
    let now = now_ms();
    let beat = |p: &PostState| ctx.journal.last_beat(&p.post.repo);
    json!({
        "posts": st.live_posts().map(|p| post_json(p, beat(p), now)).collect::<Vec<_>>(),
        "history": st.history().map(|p| post_json(p, beat(p), now)).collect::<Vec<_>>(),
    })
}

/// `tombstone`: a post whose lease is expired goes to the history; a live lease refuses it
/// (BOARD-33, BOARD-34).
fn tombstone(ctx: &Ctx, id: Option<&str>) -> Out {
    let id = id.ok_or("post is empty")?.to_string();
    let beat_of = |repo: &str| ctx.journal.last_beat(repo);
    write(ctx, |st: &State| {
        let p = st.post(&id).filter(|p| p.end.is_none()).ok_or("not found")?;
        if p.lease(now_ms(), beat_of(&p.post.repo)).live {
            return Err("the lease of this post is live: its owner is alive".into());
        }
        let mut ev = Vec::new();
        if !p.expired() {
            ev.push(Event::Expired { post: id.clone() });
        }
        ev.push(Event::Tombstone { post: id.clone(), repo: ctx.repo.clone(), session: ctx.session });
        Ok(ev)
    })?;
    Ok(json!({ "ok": true }))
}

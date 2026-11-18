//! The steps of a turn in the transcript: open a step, open its file, add its result to the
//! question (docs/zendo/interactivity.md: INTER-2, INTER-9, INTER-10).
use super::*;

impl App {
    /// The item and the step at the row of the transcript cursor; no step for the line of a run.
    fn step_at_cursor(&self) -> Option<(usize, Option<usize>)> {
        let tr = self.threads.transcript();
        tr.step_at(tr.cursor().0)
    }

    /// The step `j` of the item `item` of the thread on screen.
    fn step(&self, item: usize, j: usize) -> Option<&ToolUse> {
        match self.threads.get(&self.key())?.items.get(item)? {
            Item::Tools { uses, .. } => uses.get(j),
            _ => None,
        }
    }

    /// `o` or a click on a step: open or close it; on the line of a run, all of its steps (INTER-2).
    /// True when the cursor is on a step.
    pub(crate) fn toggle_step(&mut self) -> bool {
        let Some((item, step)) = self.step_at_cursor() else { return false };
        let key = self.key();
        let Some(Item::Tools { uses, .. }) = self.thread(&key).items.get_mut(item) else { return false };
        match step {
            Some(j) => uses.iter_mut().skip(j).take(1).for_each(|u| u.open = !u.open),
            None => {
                let open = !uses.iter().any(|u| u.open);
                uses.iter_mut().for_each(|u| u.open = open);
            }
        }
        true
    }

    /// A click in the transcript: on the file of a step, the file opens at its line (INTER-9); on
    /// another part of a step, the step opens or closes (INTER-2).
    pub(crate) fn click_step(&mut self) {
        let (row, col) = self.threads.transcript().cursor();
        let Some((place, line)) = self.step_file_at(row, col) else {
            self.toggle_step();
            return;
        };
        let path = self.cfg.root.join(&place);
        if !path.is_file() {
            return self.flash(format!("{place} is not a file here"));
        }
        self.open_file(path);
        if let Some(l) = line {
            self.reader.jump(l.saturating_sub(1) as usize);
        }
    }

    /// The file of the step at `row` and its line, when `col` is on the file in that row.
    fn step_file_at(&self, row: usize, col: usize) -> Option<(String, Option<u32>)> {
        let (item, Some(j)) = self.threads.transcript().step_at(row)? else { return None };
        let u = self.step(item, j)?;
        let place = u.info.place.clone().filter(|p| !p.is_empty())?;
        let text = self.threads.transcript().rows().get(row)?;
        let start = text[..text.find(&place)?].chars().count();
        (start..start + place.chars().count()).contains(&col).then_some((place, u.info.line))
    }

    /// `a` on a step without a selection: its title, its input and its output go to the question
    /// as a chip (INTER-10). True when the cursor is on a step.
    pub(crate) fn step_to_chat(&mut self) -> bool {
        if self.threads.transcript().selecting() {
            return false;
        }
        let Some((item, Some(j))) = self.step_at_cursor() else { return false };
        let Some(u) = self.step(item, j) else { return false };
        let mut ctx = format!("A step of the agent in this chat: {}\n", u.title);
        ctx.extend(u.info.input.iter().map(|i| format!("Input: {i}\n")));
        ctx.extend(u.info.exit.iter().map(|c| format!("Exit code: {c}\n")));
        ctx.extend(u.info.output.iter().map(|l| format!("> {l}\n")));
        let label = self.composer.add_chip(format!("{CHAT_MARK} {}", chips::text_label(&u.title)), Some(ctx), None, true);
        self.flash(format!("{} is in the chat", label.trim_matches([chips::OPEN, chips::CLOSE])));
        true
    }
}

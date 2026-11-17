
use craft_ext_core::id::ExtensionId;
use serde_json::{Value, json};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

pub const MAX_PER_PLUGIN: usize = 4;
pub const MAX_RUNNING: usize = 16;
const KEEP_FINISHED: usize = 32;

#[derive(Clone, Debug, PartialEq)]
pub enum JobState {
    Running,
    Done(Value),
    Failed(String),
    Cancelled,
}

impl JobState {
    pub fn finished(&self) -> bool {
        !matches!(self, Self::Running)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct JobInfo {
    pub id: u64,
    pub plugin: ExtensionId,
    pub contribution: String,
    pub label: String,
    pub progress: Option<f32>,
    pub note: String,
    pub cancellable: bool,
    pub state: JobState,
    pub delivered: bool,
    pub document: Option<String>,
}

impl JobInfo {
    fn clone_meta(&self) -> Self {
        Self {
            id: self.id,
            plugin: self.plugin.clone(),
            contribution: self.contribution.clone(),
            label: self.label.clone(),
            progress: self.progress,
            note: self.note.clone(),
            cancellable: self.cancellable,
            state: JobState::Running,
            delivered: self.delivered,
            document: self.document.clone(),
        }
    }

    pub fn event(&self) -> Value {
        match &self.state {
            JobState::Done(result) => json!({"event":"job","job":self.id,"ok":true,"result":result}),
            JobState::Failed(why) => json!({"event":"job","job":self.id,"ok":false,"error":why}),
            JobState::Cancelled => json!({"event":"job","job":self.id,"ok":false,"cancelled":true}),
            JobState::Running => json!({"event":"job","job":self.id,"running":true}),
        }
    }
}

pub struct Ticket {
    pub id: u64,
    pub cancel: Arc<AtomicBool>,
    board: Arc<JobBoard>,
}

impl std::fmt::Debug for Ticket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ticket").field("id", &self.id).finish_non_exhaustive()
    }
}

impl Ticket {
    pub fn progress(&self, value: Option<f32>, note: &str) {
        self.board.update(self.id, |j| {
            j.progress = value.map(|v| if v.is_finite() { v.clamp(0.0, 1.0) } else { 0.0 });
            if !note.is_empty() {
                j.note = note.chars().take(200).collect();
            }
        });
    }
    pub fn finish(&self, state: JobState) {
        let state = if self.cancel.load(Ordering::Relaxed) && !matches!(state, JobState::Done(_)) { JobState::Cancelled } else { state };
        self.board.update(self.id, |j| j.state = state);
    }
}

#[derive(Default)]
pub struct JobBoard {
    jobs: Mutex<Vec<(JobInfo, Arc<AtomicBool>)>>,
    next: AtomicU64,
}

impl JobBoard {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    fn lock(&self) -> MutexGuard<'_, Vec<(JobInfo, Arc<AtomicBool>)>> {
        self.jobs.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn update(&self, id: u64, f: impl FnOnce(&mut JobInfo)) {
        if let Some((j, _)) = self.lock().iter_mut().find(|(j, _)| j.id == id) {
            f(j);
        }
    }

    pub fn admit(self: &Arc<Self>, plugin: &ExtensionId, contribution: &str, label: &str, cancellable: bool, document: Option<&str>) -> Result<Ticket, String> {
        let mut jobs = self.lock();
        let running = jobs.iter().filter(|(j, _)| !j.state.finished());
        let (mine, all) = running.fold((0, 0), |(m, a), (j, _)| (m + usize::from(&j.plugin == plugin), a + 1));
        if mine >= MAX_PER_PLUGIN {
            return Err(format!("at most {MAX_PER_PLUGIN} jobs per plugin run at once"));
        }
        if all >= MAX_RUNNING {
            return Err("too many background jobs are running; try again when one finishes".into());
        }
        let id = self.next.fetch_add(1, Ordering::Relaxed) + 1;
        let cancel = Arc::new(AtomicBool::new(false));
        let info = JobInfo {
            id,
            plugin: plugin.clone(),
            contribution: contribution.to_owned(),
            label: label.chars().take(120).collect(),
            progress: None,
            note: String::new(),
            cancellable,
            state: JobState::Running,
            delivered: false,
            document: document.map(str::to_owned),
        };
        jobs.push((info, cancel.clone()));
        let finished: Vec<u64> = jobs.iter().filter(|(j, _)| j.delivered).map(|(j, _)| j.id).collect();
        if finished.len() > KEEP_FINISHED {
            let drop: Vec<u64> = finished[..finished.len() - KEEP_FINISHED].to_vec();
            jobs.retain(|(j, _)| !drop.contains(&j.id));
        }
        Ok(Ticket { id, cancel, board: self.clone() })
    }

    pub fn list(&self) -> Vec<JobInfo> {
        let mut v: Vec<JobInfo> = self
            .lock()
            .iter()
            .map(|(j, _)| JobInfo { state: if matches!(j.state, JobState::Done(_)) { JobState::Done(Value::Null) } else { j.state.clone() }, ..j.clone_meta() })
            .collect();
        v.reverse();
        v
    }

    pub fn owner(&self, id: u64) -> Option<ExtensionId> {
        self.lock().iter().find(|(j, _)| j.id == id).map(|(j, _)| j.plugin.clone())
    }

    pub fn running(&self) -> usize {
        self.lock().iter().filter(|(j, _)| !j.state.finished()).count()
    }

    pub fn cancel(&self, id: u64, force: bool) -> bool {
        self.lock().iter().find(|(j, _)| j.id == id && !j.state.finished() && (j.cancellable || force)).is_some_and(|(_, c)| {
            c.store(true, Ordering::Relaxed);
            true
        })
    }

    pub fn cancel_plugin(&self, plugin: &ExtensionId) {
        for (j, c) in self.lock().iter() {
            if &j.plugin == plugin && !j.state.finished() {
                c.store(true, Ordering::Relaxed);
            }
        }
    }

    pub fn deliver(&self, ready: impl Fn(&JobInfo) -> bool, mut send: impl FnMut(&JobInfo) -> bool) {
        let candidates: Vec<JobInfo> = self
            .lock()
            .iter()
            .filter(|(j, _)| j.state.finished() && !j.delivered && ready(j))
            .map(|(j, _)| JobInfo { state: j.state.clone(), ..j.clone_meta() })
            .collect();
        for job in candidates {
            if send(&job) {
                self.update(job.id, |j| {
                    j.delivered = true;
                    if let JobState::Done(v) = &mut j.state {
                        *v = Value::Null;
                    }
                });
            }
        }
    }

    pub fn cancel_document(&self, document: &str) {
        for (j, c) in self.lock().iter_mut() {
            if j.document.as_deref() == Some(document) {
                c.store(true, Ordering::Relaxed);
                if j.state.finished() {
                    j.delivered = true;
                    j.state = JobState::Cancelled;
                }
            }
        }
    }

    pub fn clear_finished(&self) {
        self.lock().retain(|(j, _)| !(j.state.finished() && j.delivered));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn take(board: &JobBoard) -> Vec<JobInfo> {
        let mut out = Vec::new();
        board.deliver(|_| true, |j| {
            out.push(j.clone());
            true
        });
        out
    }

    #[test]
    fn a_result_waits_for_its_document_and_is_delivered_only_when_accepted() {
        let board = JobBoard::new();
        let p = id("io.github.a.b");
        let a = board.admit(&p, "c", "A", true, Some("doc-a")).unwrap();
        let b = board.admit(&p, "c", "B", true, Some("doc-b")).unwrap();
        a.finish(JobState::Done(json!(1)));
        b.finish(JobState::Done(json!(2)));
        let mut seen = Vec::new();
        board.deliver(|j| j.document.as_deref() == Some("doc-b"), |j| {
            seen.push(j.event());
            false
        });
        assert_eq!(seen, vec![json!({"event":"job","job":2,"ok":true,"result":2})]);
        assert_eq!(take(&board).len(), 2, "a refused hand-over stays pending");
        let c = board.admit(&p, "c", "C", true, Some("doc-c")).unwrap();
        board.cancel_document("doc-c");
        assert!(c.cancel.load(Ordering::Relaxed));
    }

    fn id(s: &str) -> ExtensionId {
        ExtensionId::new(s).unwrap()
    }

    #[test]
    fn limits_cancel_and_delivery_follow_the_job_lifecycle() {
        let board = JobBoard::new();
        let p = id("io.github.a.b");
        let tickets: Vec<Ticket> = (0..MAX_PER_PLUGIN).map(|_| board.admit(&p, "c", "Render", true, None).unwrap()).collect();
        assert_eq!(board.admit(&p, "c", "Render", true, None).unwrap_err(), "at most 4 jobs per plugin run at once");
        assert!(board.admit(&id("io.github.a.other"), "c", "x", true, None).is_ok());

        tickets[0].progress(Some(1.5), "frame 3");
        let first = board.list().into_iter().find(|j| j.id == tickets[0].id).unwrap();
        assert_eq!((first.progress, first.note.as_str()), (Some(1.0), "frame 3"));

        assert!(board.cancel(tickets[1].id, false));
        tickets[0].finish(JobState::Done(json!({"frames": 3})));
        tickets[1].finish(JobState::Failed("stopped".into()));
        let done = take(&board);
        assert_eq!(done.len(), 2);
        assert_eq!(done[0].event(), json!({"event":"job","job":1,"ok":true,"result":{"frames":3}}));
        assert_eq!(done[1].event(), json!({"event":"job","job":2,"ok":false,"cancelled":true}));
        assert!(take(&board).is_empty(), "each job is delivered once");

        board.cancel_plugin(&p);
        assert!(tickets[2..].iter().all(|t| t.cancel.load(Ordering::Relaxed)));
    }

    #[test]
    fn a_job_that_is_not_cancellable_ignores_cancel_unless_forced() {
        let board = JobBoard::new();
        let t = board.admit(&id("io.github.a.b"), "c", "Save", false, None).unwrap();
        assert!(!board.cancel(t.id, false));
        assert!(board.cancel(t.id, true));
    }
}

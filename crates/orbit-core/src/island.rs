//! Orbit Island — pure state machine for the interactive pill UI.
//!
//! The core owns notification lifecycle and focus timers; the frontend only
//! renders state and forwards user intents (expand/collapse/dismiss/start).
//! This keeps every behavior unit-testable without a display server.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

const MAX_QUEUE: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Priority {
    Low,
    Normal,
    High,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum IslandContent {
    /// Generic short-lived toast (e.g. "Ayarlar kaydedildi").
    Toast { message: String },
    /// Clipboard item was stored (Recall). Never includes the content itself unless granted.
    ClipboardStored { preview: Option<String> },
    /// A screenshot landed (CleanShot).
    ScreenshotCaptured { path: String },
    /// File transfer progress (GhostDrop).
    FileTransfer {
        label: String,
        bytes_done: u64,
        bytes_total: u64,
    },
    /// Workspace switch (SnapSpace).
    WorkspaceSwitched { workspace: String },
    /// Coarse system load snapshot (AppPulse). Emitted by the sampler, not invented by the UI.
    SystemLoad { cpu_percent: f32, ram_percent: f32 },
    /// A focus/Pomodoro-style timer finished.
    TimerDone { label: String },
}

/// Currently-playing media reported by the host (e.g. MPRIS on KDE/Linux).
/// The island never invents this; the host pushes it and clears it when idle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NowPlaying {
    /// MPRIS player identity (e.g. "spotify_client").
    pub player: String,
    pub title: String,
    pub artist: String,
    /// Seconds into the track at push time — hosts refresh as it plays.
    pub position_secs: u64,
    /// Track length in seconds, if known.
    pub length_secs: Option<u64>,
    pub playing: bool,
    /// Media volume 0.0–1.0 when the player exposes it (MPRIS `Volume`); `None` otherwise — never guessed.
    pub volume: Option<f32>,
}

/// Coarse machine load for the island. Sampled by the host from local sources
/// (procfs on Linux); never leaves the device.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SystemStats {
    pub cpu_percent: f32,
    pub ram_percent: f32,
    pub ram_used_mb: u64,
    pub ram_total_mb: u64,
}

/// One in-flight or finished download/transfer shown in the island.
/// Browser downloads and Orbit transfers share this shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadTask {
    /// Stable host-chosen key (e.g. absolute path for watched downloads,
    /// or the GhostDrop transfer id).
    pub id: String,
    /// Display label (file name). Hosts clamp to short text.
    pub label: String,
    pub received_bytes: u64,
    /// `None` when the total is unknowable (e.g. chunked HTTP) — never guessed.
    pub total_bytes: Option<u64>,
    pub done: bool,
}

/// A file the user dropped onto the island. Listing only — Orbit never
/// copies or moves anything without an explicit action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DroppedFile {
    pub path: String,
    pub bytes: u64,
}

/// How many completed download tasks to keep for the "recent" list.
const MAX_KEEP_DONE_DOWNLOADS: usize = 4;
/// Upper bound on tracked in-flight downloads.
const MAX_TRACK_DOWNLOADS: usize = 8;
/// Upper bound on remembered dropped files.
const MAX_DROPPED_FILES: usize = 16;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IslandNotification {
    pub id: u64,
    pub content: IslandContent,
    pub priority: Priority,
    /// Milliseconds since UNIX epoch at creation.
    pub created_at_ms: i64,
    /// Auto-dismiss delay; `None` keeps the notification until dismissed.
    pub auto_dismiss_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FocusTimer {
    pub label: String,
    pub ends_at_ms: i64,
    pub duration_secs: u64,
    pub paused: bool,
    /// Remaining seconds captured at pause time.
    pub remaining_secs: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IslandView {
    /// Pill showing the highest-priority item (or idle glyph).
    Collapsed,
    /// Expanded panel with details + quick actions.
    Expanded,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IslandSnapshot {
    pub view: IslandView,
    pub active: Option<IslandNotification>,
    pub queued: usize,
    pub timer: Option<FocusTimer>,
    pub timer_remaining_secs: Option<u64>,
    pub now_playing: Option<NowPlaying>,
    /// In-flight tasks first, then up to [`MAX_KEEP_DONE_DOWNLOADS`] finished ones.
    pub downloads: Vec<DownloadTask>,
    pub downloads_active: usize,
    /// Files dropped on the island (listing only; nothing is moved/copied).
    pub dropped_files: Vec<DroppedFile>,
    /// Coarse local CPU/RAM load; `None` when the sampler is off or unsupported on this platform.
    pub system: Option<SystemStats>,
}

/// Drives the island. Not async; call [`IslandManager::tick`] from a timer.
pub struct IslandManager {
    view: IslandView,
    active: Option<IslandNotification>,
    queue: VecDeque<IslandNotification>,
    next_id: u64,
    timer: Option<FocusTimer>,
    now_playing: Option<NowPlaying>,
    downloads: Vec<DownloadTask>,
    dropped_files: Vec<DroppedFile>,
    system: Option<SystemStats>,
}

impl Default for IslandManager {
    fn default() -> Self {
        Self::new()
    }
}

impl IslandManager {
    pub fn new() -> Self {
        Self {
            view: IslandView::Collapsed,
            active: None,
            queue: VecDeque::new(),
            next_id: 1,
            timer: None,
            now_playing: None,
            downloads: Vec::new(),
            dropped_files: Vec::new(),
            system: None,
        }
    }

    fn now_ms() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64
    }

    /// Push a notification. High priority preempts the current one (current
    /// returns to the front of the queue). Returns the assigned id.
    pub fn push(&mut self, content: IslandContent, priority: Priority, auto_dismiss_ms: Option<u64>) -> u64 {
        let notif = IslandNotification {
            id: self.next_id,
            content,
            priority,
            created_at_ms: Self::now_ms(),
            auto_dismiss_ms,
        };
        let id = notif.id;
        self.next_id += 1;

        match &self.active {
            None => self.active = Some(notif),
            Some(current) => {
                if priority > current.priority {
                    let current = self.active.take().expect("active checked");
                    self.queue.push_front(current);
                    self.active = Some(notif);
                } else {
                    if self.queue.len() >= MAX_QUEUE {
                        self.queue.pop_front(); // drop oldest low-priority item
                    }
                    self.queue.push_back(notif);
                }
            }
        }
        id
    }

    /// Advance time. Expires auto-dismiss notifications and fires timer
    /// completion. Returns a snapshot plus whether anything changed.
    pub fn tick(&mut self) -> IslandSnapshot {
        let now = Self::now_ms();
        // Expire active notification if due.
        if let Some(active) = &self.active {
            if let Some(ms) = active.auto_dismiss_ms {
                if now - active.created_at_ms >= ms as i64 {
                    self.advance();
                }
            }
        }
        // Timer completion → emit a High priority TimerDone notification.
        if let Some(timer) = &self.timer {
            if !timer.paused && now >= timer.ends_at_ms {
                let label = timer.label.clone();
                self.timer = None;
                self.push(IslandContent::TimerDone { label }, Priority::High, Some(10_000));
            }
        }
        self.snapshot()
    }

    /// Dismiss a specific notification id (active or queued).
    pub fn dismiss(&mut self, id: u64) {
        if self.active.as_ref().map(|n| n.id) == Some(id) {
            self.advance();
            return;
        }
        self.queue.retain(|n| n.id != id);
    }

    /// Dismiss whatever is currently shown and pull the next queued item.
    pub fn dismiss_active(&mut self) {
        self.advance();
    }

    fn advance(&mut self) {
        self.active = self.queue.pop_front();
    }

    pub fn toggle_expand(&mut self) -> IslandView {
        self.view = match self.view {
            IslandView::Collapsed => IslandView::Expanded,
            IslandView::Expanded => IslandView::Collapsed,
        };
        self.view
    }

    pub fn set_expanded(&mut self, expanded: bool) {
        self.view = if expanded {
            IslandView::Expanded
        } else {
            IslandView::Collapsed
        };
    }

    pub fn start_timer(&mut self, label: &str, duration_secs: u64) {
        let now = Self::now_ms();
        self.timer = Some(FocusTimer {
            label: label.to_string(),
            ends_at_ms: now + (duration_secs as i64) * 1000,
            duration_secs,
            paused: false,
            remaining_secs: duration_secs,
        });
    }

    pub fn cancel_timer(&mut self) {
        self.timer = None;
    }

    pub fn toggle_timer_pause(&mut self) {
        let now = Self::now_ms();
        if let Some(timer) = &mut self.timer {
            if timer.paused {
                timer.ends_at_ms = now + (timer.remaining_secs as i64) * 1000;
                timer.paused = false;
            } else {
                let remaining = ((timer.ends_at_ms - now).max(0) / 1000) as u64;
                timer.remaining_secs = remaining;
                timer.paused = true;
            }
        }
    }

    /// Host pushes (or clears with `None`) the currently-playing media item.
    pub fn set_now_playing(&mut self, now_playing: Option<NowPlaying>) {
        self.now_playing = now_playing;
    }

    /// Host pushes (or clears with `None`) the coarse local system load sample.
    pub fn set_system(&mut self, system: Option<SystemStats>) {
        self.system = system;
    }

    /// Insert or update one in-flight/finished download. Label is trimmed to 80 chars.
    /// When the in-flight table is full, the oldest *done* entry is evicted; if none
    /// are done, the new task is refused rather than silently evicting live state.
    pub fn upsert_download(&mut self, task: DownloadTask) {
        let label = task.label.trim();
        if label.is_empty() {
            return;
        }
        let clamped_label: String = label.chars().take(80).collect();
        let mut task = task;
        task.label = clamped_label;
        if let Some(slot) = self.downloads.iter_mut().find(|d| d.id == task.id) {
            *slot = task;
            return;
        }
        let inflight = self.downloads.iter().filter(|d| !d.done).count();
        if inflight >= MAX_TRACK_DOWNLOADS {
            match self.downloads.iter().position(|d| d.done) {
                Some(pos) => {
                    self.downloads.remove(pos);
                }
                None => return, // refuse rather than evict a live transfer
            }
        }
        self.downloads.push(task);
        self.trim_downloads();
    }

    pub fn mark_download_done(&mut self, id: &str) {
        if let Some(d) = self.downloads.iter_mut().find(|d| d.id == id) {
            d.done = true;
        }
        self.trim_downloads();
    }

    pub fn remove_download(&mut self, id: &str) {
        self.downloads.retain(|d| d.id != id);
    }

    fn trim_downloads(&mut self) {
        let mut done_kept = 0;
        self.downloads.retain(|d| {
            if d.done {
                done_kept += 1;
                done_kept <= MAX_KEEP_DONE_DOWNLOADS
            } else {
                true
            }
        });
    }

    /// Remember a file dropped on the island (listing only, nothing is copied/moved).
    pub fn add_dropped_file(&mut self, file: DroppedFile) {
        if file.path.trim().is_empty() {
            return;
        }
        self.dropped_files.push(file);
        if self.dropped_files.len() > MAX_DROPPED_FILES {
            let excess = self.dropped_files.len() - MAX_DROPPED_FILES;
            self.dropped_files.drain(0..excess);
        }
    }

    pub fn clear_dropped_files(&mut self) {
        self.dropped_files.clear();
    }

    pub fn snapshot(&self) -> IslandSnapshot {
        let now = Self::now_ms();
        let timer_remaining_secs = self.timer.as_ref().map(|t| {
            if t.paused {
                t.remaining_secs
            } else {
                ((t.ends_at_ms - now).max(0) / 1000) as u64
            }
        });
        IslandSnapshot {
            view: self.view,
            active: self.active.clone(),
            queued: self.queue.len(),
            timer: self.timer.clone(),
            timer_remaining_secs,
            now_playing: self.now_playing.clone(),
            downloads: self.downloads.clone(),
            downloads_active: self.downloads.iter().filter(|d| !d.done).count(),
            dropped_files: self.dropped_files.clone(),
            system: self.system,
        }
    }
}

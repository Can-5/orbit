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
}

/// Drives the island. Not async; call [`IslandManager::tick`] from a timer.
pub struct IslandManager {
    view: IslandView,
    active: Option<IslandNotification>,
    queue: VecDeque<IslandNotification>,
    next_id: u64,
    timer: Option<FocusTimer>,
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
        }
    }
}

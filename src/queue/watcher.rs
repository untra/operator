#![allow(dead_code)]

use anyhow::Result;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

pub enum QueueEvent {
    Added(PathBuf),
    Removed(PathBuf),
    Modified(PathBuf),
}

pub struct QueueWatcher {
    _watcher: RecommendedWatcher,
    receiver: Receiver<Result<Event, notify::Error>>,
}

impl QueueWatcher {
    pub fn new(tickets_path: PathBuf) -> Result<Self> {
        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default().with_poll_interval(Duration::from_secs(1)),
        )?;

        // Watch the queue directory
        let queue_path = tickets_path.join("queue");
        if queue_path.exists() {
            watcher.watch(&queue_path, RecursiveMode::NonRecursive)?;
        }

        // Watch in-progress for status changes
        let in_progress_path = tickets_path.join("in-progress");
        if in_progress_path.exists() {
            watcher.watch(&in_progress_path, RecursiveMode::NonRecursive)?;
        }

        Ok(Self {
            _watcher: watcher,
            receiver: rx,
        })
    }

    /// Poll for queue events (non-blocking)
    pub fn poll(&self) -> Option<QueueEvent> {
        match self.receiver.try_recv() {
            Ok(Ok(event)) => self.classify_event(event),
            _ => None,
        }
    }

    /// Wait for next queue event (blocking)
    pub fn next(&self) -> Option<QueueEvent> {
        match self.receiver.recv() {
            Ok(Ok(event)) => self.classify_event(event),
            _ => None,
        }
    }

    fn classify_event(&self, event: Event) -> Option<QueueEvent> {
        use notify::EventKind;

        let path = event.paths.first()?.clone();

        // Only care about markdown files
        if path.extension().is_none_or(|e| e != "md") {
            return None;
        }

        match event.kind {
            EventKind::Create(_) => Some(QueueEvent::Added(path)),
            EventKind::Remove(_) => Some(QueueEvent::Removed(path)),
            EventKind::Modify(_) => Some(QueueEvent::Modified(path)),
            _ => None,
        }
    }
}

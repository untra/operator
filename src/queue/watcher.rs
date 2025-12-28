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

    /// Classify a filesystem event into a QueueEvent.
    /// Made pub(crate) for testing.
    pub(crate) fn classify_event(&self, event: Event) -> Option<QueueEvent> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::{AccessKind, CreateKind, DataChange, ModifyKind, RemoveKind};
    use notify::EventKind;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Create a minimal watcher for testing (bypasses actual file watching)
    fn make_test_watcher() -> QueueWatcher {
        // Create a temp directory that exists
        let temp_dir = TempDir::new().unwrap();
        let queue_path = temp_dir.path().join("queue");
        std::fs::create_dir_all(&queue_path).unwrap();

        // Create the watcher (it will watch the temp directory)
        // Note: This watcher won't receive real events, but we can test classify_event
        let tickets_path = temp_dir.keep();
        QueueWatcher::new(tickets_path).unwrap()
    }

    fn make_event(kind: EventKind, path: PathBuf) -> Event {
        Event {
            kind,
            paths: vec![path],
            attrs: Default::default(),
        }
    }

    #[test]
    fn test_classify_event_create_markdown_returns_added() {
        let watcher = make_test_watcher();
        let path = PathBuf::from("/tickets/queue/20241225-test.md");
        let event = make_event(EventKind::Create(CreateKind::File), path.clone());

        let result = watcher.classify_event(event);

        assert!(matches!(result, Some(QueueEvent::Added(p)) if p == path));
    }

    #[test]
    fn test_classify_event_remove_markdown_returns_removed() {
        let watcher = make_test_watcher();
        let path = PathBuf::from("/tickets/queue/20241225-test.md");
        let event = make_event(EventKind::Remove(RemoveKind::File), path.clone());

        let result = watcher.classify_event(event);

        assert!(matches!(result, Some(QueueEvent::Removed(p)) if p == path));
    }

    #[test]
    fn test_classify_event_modify_markdown_returns_modified() {
        let watcher = make_test_watcher();
        let path = PathBuf::from("/tickets/queue/20241225-test.md");
        let event = make_event(
            EventKind::Modify(ModifyKind::Data(DataChange::Any)),
            path.clone(),
        );

        let result = watcher.classify_event(event);

        assert!(matches!(result, Some(QueueEvent::Modified(p)) if p == path));
    }

    #[test]
    fn test_classify_event_non_markdown_returns_none() {
        let watcher = make_test_watcher();
        let path = PathBuf::from("/tickets/queue/config.json");
        let event = make_event(EventKind::Create(CreateKind::File), path);

        let result = watcher.classify_event(event);

        assert!(result.is_none());
    }

    #[test]
    fn test_classify_event_txt_extension_returns_none() {
        let watcher = make_test_watcher();
        let path = PathBuf::from("/tickets/queue/notes.txt");
        let event = make_event(EventKind::Create(CreateKind::File), path);

        let result = watcher.classify_event(event);

        assert!(result.is_none());
    }

    #[test]
    fn test_classify_event_no_extension_returns_none() {
        let watcher = make_test_watcher();
        let path = PathBuf::from("/tickets/queue/README");
        let event = make_event(EventKind::Create(CreateKind::File), path);

        let result = watcher.classify_event(event);

        assert!(result.is_none());
    }

    #[test]
    fn test_classify_event_empty_paths_returns_none() {
        let watcher = make_test_watcher();
        let event = Event {
            kind: EventKind::Create(CreateKind::File),
            paths: vec![],
            attrs: Default::default(),
        };

        let result = watcher.classify_event(event);

        assert!(result.is_none());
    }

    #[test]
    fn test_classify_event_access_event_returns_none() {
        let watcher = make_test_watcher();
        let path = PathBuf::from("/tickets/queue/test.md");
        let event = make_event(EventKind::Access(AccessKind::Read), path);

        let result = watcher.classify_event(event);

        assert!(result.is_none());
    }
}

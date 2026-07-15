use crate::ipc::RasCoreEvent;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{self, Receiver};

#[cfg(test)]
mod tests;

pub struct FsWatcher {
    _watcher: RecommendedWatcher,
    rx: Receiver<RasCoreEvent>,
}

impl FsWatcher {
    /// Creates a new `FsWatcher` watching the specified directory recursively.
    ///
    /// # Errors
    ///
    /// Returns an error if the watcher fails to initialize or start watching.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let (tx, rx) = mpsc::channel();

        let mut watcher =
            notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    let change_type = match event.kind {
                        EventKind::Create(_) => Some("create".to_string()),
                        EventKind::Modify(_) => Some("modify".to_string()),
                        EventKind::Remove(_) => Some("remove".to_string()),
                        _ => None,
                    };

                    if let Some(ct) = change_type {
                        for p in event.paths {
                            let ev = RasCoreEvent::FileChanged {
                                path: p,
                                change_type: ct.clone(),
                            };
                            let _ = tx.send(ev);
                        }
                    }
                }
            })
            .map_err(|e| format!("Failed to create watcher: {e}"))?;

        watcher
            .watch(path.as_ref(), RecursiveMode::Recursive)
            .map_err(|e| format!("Failed to watch path: {e}"))?;

        Ok(Self {
            _watcher: watcher,
            rx,
        })
    }

    /// Try to receive the next file change event without blocking.
    ///
    /// # Errors
    ///
    /// Returns a `TryRecvError` if no event is available or if channel is disconnected.
    pub fn try_recv(&self) -> Result<RasCoreEvent, mpsc::TryRecvError> {
        self.rx.try_recv()
    }
}

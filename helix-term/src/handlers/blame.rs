use std::{mem, time::Duration};

use helix_event::register_hook;
use helix_loader::workspace_trust::TrustQuery;
use helix_vcs::FileBlame;
use helix_view::{
    events::{DocumentDidOpen, EditorConfigDidChange},
    handlers::{BlameEvent, Handlers},
    DocumentId,
};
use tokio::time::Instant;

use crate::job;

/// Handles showing Git Blame for a single document
#[derive(Default)]
pub struct BlameHandler {
    /// Computed blame for the file. This is in an `Option` because we
    /// need to be able to `mem::take` it, as we will need an owned instance
    /// when we only have access to `&mut self`
    file_blame: Option<anyhow::Result<FileBlame>>,
    /// Document for which we will update the blame
    doc_id: DocumentId,
    /// If `Some`, when blame is obtained for the file, the user will be notified
    show_blame_for_line_in_statusline: Option<u32>,
}

impl helix_event::AsyncHook for BlameHandler {
    type Event = BlameEvent;

    fn handle_event(
        &mut self,
        event: Self::Event,
        _timeout: Option<tokio::time::Instant>,
    ) -> Option<tokio::time::Instant> {
        self.doc_id = event.doc_id;
        self.show_blame_for_line_in_statusline = event.line;
        self.file_blame = Some(FileBlame::try_new(event.path, event.trust_full));
        Some(Instant::now() + Duration::from_millis(50))
    }

    fn finish_debounce(&mut self) {
        let doc_id = self.doc_id;
        let line_blame = self.show_blame_for_line_in_statusline;
        let result = mem::take(&mut self.file_blame);
        if let Some(result) = result {
            tokio::spawn(async move {
                job::dispatch(move |editor, _| {
                    let Some(doc) = editor.document_mut(doc_id) else {
                        return;
                    };
                    doc.file_blame = Some(result);
                    if !editor.config().inline_blame.auto_fetch {
                        if let Some(line) = line_blame {
                            crate::commands::blame_line_impl(editor, doc_id, line);
                        } else {
                            editor.set_status("Blame for this file is now available")
                        }
                    }
                })
                .await;
            });
        }
    }
}

pub(super) fn register_hooks(handlers: &Handlers) {
    let tx = handlers.blame.clone();
    register_hook!(move |event: &mut DocumentDidOpen<'_>| {
        if event.editor.config().inline_blame.auto_fetch {
            let trust_full = event
                .editor
                .document(event.doc)
                .map(|doc| {
                    event
                        .editor
                        .workspace_trust
                        .query(doc.workspace_root(), TrustQuery::Git)
                        .is_trusted()
                })
                .unwrap_or(false);
            helix_event::send_blocking(
                &tx,
                BlameEvent {
                    path: event.path.to_path_buf(),
                    doc_id: event.doc,
                    line: None,
                    trust_full,
                },
            );
        }
        Ok(())
    });
    let tx = handlers.blame.clone();
    register_hook!(move |event: &mut EditorConfigDidChange<'_>| {
        let has_enabled_inline_blame = !event.old_config.inline_blame.auto_fetch
            && event.editor.config().inline_blame.auto_fetch;

        if has_enabled_inline_blame {
            // request blame for all documents, since any of them could have
            // outdated blame
            let workspace_trust = &event.editor.workspace_trust;
            let doc_paths: Vec<_> = event
                .editor
                .documents()
                .filter_map(|doc| {
                    doc.path().map(|path| {
                        let trust_full = workspace_trust
                            .query(doc.workspace_root(), TrustQuery::Git)
                            .is_trusted();
                        (path.to_path_buf(), doc.id(), trust_full)
                    })
                })
                .collect();
            for (path, doc_id, trust_full) in doc_paths {
                helix_event::send_blocking(
                    &tx,
                    BlameEvent {
                        path,
                        doc_id,
                        line: None,
                        trust_full,
                    },
                );
            }
        }
        Ok(())
    });
}

//! ITfThreadMgrEventSink implementation — tracks document focus changes.

use windows::Win32::UI::TextServices::*;
use windows::core::*;

use crate::tsf::text_input_processor::AfrimDesktopTextService_Impl;

impl ITfThreadMgrEventSink_Impl for AfrimDesktopTextService_Impl {
    /// Called when a document (context) gets focus.
    fn OnInitDocumentMgr(&self, _pdim: Option<&ITfDocumentMgr>) -> Result<()> {
        Ok(())
    }

    /// Called when a document manager is being removed.
    fn OnUninitDocumentMgr(&self, _pdim: Option<&ITfDocumentMgr>) -> Result<()> {
        Ok(())
    }

    /// Called when focus changes to a new document.
    fn OnSetFocus(
        &self,
        _pdimfocus: Option<&ITfDocumentMgr>,
        _pdimprevfocus: Option<&ITfDocumentMgr>,
    ) -> Result<()> {
        let mut inner = match self.inner.try_borrow_mut() {
            Ok(inner) => inner,
            Err(_) => {
                // Reentrant call (e.g., triggered during ActivateEx's AdviseSink)
                tracing::debug!("OnSetFocus: reentrant borrow, skipping");
                return Ok(());
            }
        };

        // Reset engine state when focus changes to avoid stale state
        inner.engine.reset();
        inner.composition = None;

        // Hide candidate window
        if let Some(ref cw) = inner.candidate_window {
            cw.borrow_mut().hide();
        }

        Ok(())
    }

    /// Called when a context is pushed onto the document manager.
    fn OnPushContext(&self, _pic: Option<&ITfContext>) -> Result<()> {
        Ok(())
    }

    /// Called when a context is popped from the document manager.
    fn OnPopContext(&self, _pic: Option<&ITfContext>) -> Result<()> {
        Ok(())
    }
}

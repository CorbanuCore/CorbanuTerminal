const MAX_PENDING_OUTPUT_TEXT_BYTES: usize = 1024 * 1024;

#[derive(Debug, Default)]
pub(super) struct PendingOutputText {
    item_id: Option<String>,
    text: String,
}

impl PendingOutputText {
    pub(super) fn push(&mut self, item_id: Option<String>, delta: String) -> Result<(), String> {
        if delta.is_empty() {
            return Ok(());
        }

        match (self.item_id.as_deref(), item_id.as_deref()) {
            (Some(pending_item_id), Some(delta_item_id)) if pending_item_id != delta_item_id => {
                return Err(format!(
                    "received interleaved output text deltas before their items were active: \
                     pending item `{pending_item_id}`, new item `{delta_item_id}`"
                ));
            }
            (None, Some(_)) if !self.text.is_empty() => self.item_id = item_id,
            (None, _) if self.text.is_empty() => self.item_id = item_id,
            _ => {}
        }

        let next_len = self
            .text
            .len()
            .checked_add(delta.len())
            .ok_or_else(|| "pending output text length overflowed".to_string())?;
        if next_len > MAX_PENDING_OUTPUT_TEXT_BYTES {
            return Err(format!(
                "output text arrived before its item and exceeded the {MAX_PENDING_OUTPUT_TEXT_BYTES} byte recovery limit"
            ));
        }
        self.text.push_str(&delta);
        Ok(())
    }

    pub(super) fn take_for_item(&mut self, item_id: &str) -> Option<String> {
        if self.text.is_empty()
            || self
                .item_id
                .as_deref()
                .is_some_and(|pending_item_id| pending_item_id != item_id)
        {
            return None;
        }
        self.item_id = None;
        Some(std::mem::take(&mut self.text))
    }

    pub(super) fn discard_for_completed_item(&mut self, item_id: &str) -> usize {
        if self
            .item_id
            .as_deref()
            .is_some_and(|pending_item_id| pending_item_id != item_id)
        {
            return 0;
        }
        let discarded_len = self.text.len();
        self.item_id = None;
        self.text.clear();
        discarded_len
    }

    pub(super) fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub(super) fn len(&self) -> usize {
        self.text.len()
    }

    pub(super) fn item_id(&self) -> Option<&str> {
        self.item_id.as_deref()
    }
}

#[cfg(test)]
#[path = "output_text_stream_tests.rs"]
mod tests;

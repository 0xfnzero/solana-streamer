use crate::streaming::event_parser::common::{
    types::EventType, ACCOUNT_EVENT_TYPES, BLOCK_EVENT_TYPES,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct EventTypeFilter {
    pub include: Vec<EventType>,
}

impl EventTypeFilter {
    pub fn include_transaction_event(&self) -> bool {
        self.include
            .iter()
            .any(|event| !ACCOUNT_EVENT_TYPES.contains(event) && !BLOCK_EVENT_TYPES.contains(event))
    }

    pub fn include_account_event(&self) -> bool {
        self.include.iter().any(|event| ACCOUNT_EVENT_TYPES.contains(event))
    }

    pub fn include_block_event(&self) -> bool {
        self.include.iter().any(|event| BLOCK_EVENT_TYPES.contains(event))
    }
}

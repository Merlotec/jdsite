
use crate::{db, define_uuid_key};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum SectionState {
    InProgress,
    InReview,
    Completed,
}

impl Default for SectionState {
    fn default() -> Self {
        SectionState::InProgress
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Section {
    description: String,
    pre_text: String,
    post_text: String,
    assets: Vec<String>,
    state: SectionState,
}

define_uuid_key!(SectionKey);

pub type SectionDb = db::Database<SectionKey, Section>;
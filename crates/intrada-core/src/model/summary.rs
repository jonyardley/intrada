use serde::{Deserialize, Serialize};

use super::SetlistEntryView;
use crate::analytics::ScoreChange;
use crate::domain::session::CompletionStatus;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct SummaryView {
    pub total_duration_display: String,
    pub completion_status: CompletionStatus,
    pub notes: Option<String>,
    pub entries: Vec<SetlistEntryView>,
    pub session_score: Option<u8>,
    pub completed_count: usize,
    /// The largest rise this week among this session's items, by item id.
    pub top_mover: Option<ScoreChange>,
}

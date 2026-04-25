use chrono::{DateTime, Utc};

use brainy_domain::{Guid, cells::entities::repetition::Repetition};

use crate::infrastructure::repositories::sqlite::sqlite_rows::cell_row::state_sqlite_impls::StateSqlite;

pub struct RepetitionRow {
    pub id: Guid,
    pub created_date: DateTime<Utc>,
    pub modified_date: DateTime<Utc>,
    pub file_id: Guid,
    pub cell_id: Guid,
    pub due: DateTime<Utc>,
    pub stability: f64,
    pub difficulty: f64,
    pub elapsed_days: i64,
    pub scheduled_days: i64,
    pub reps: i64,
    pub lapses: i64,
    pub state: StateSqlite,
    pub last_review: Option<DateTime<Utc>>,
    pub additional_content: Option<String>,
}

impl From<RepetitionRow> for Repetition {
    fn from(value: RepetitionRow) -> Self {
        Repetition::new_unchecked(
            value.id,
            value.created_date,
            value.modified_date,
            value.file_id,
            value.cell_id,
            value.due,
            value.stability,
            value.difficulty,
            value.elapsed_days,
            value.scheduled_days,
            value.reps,
            value.lapses,
            value.state.into(),
            value.last_review,
            value.additional_content,
        )
    }
}

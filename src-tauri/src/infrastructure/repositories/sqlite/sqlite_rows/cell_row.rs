use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};

use brainy_domain::{
    Guid,
    cells::entities::{
        cell::{Cell, CellType},
        repetition::{Repetition, State},
    },
};

use crate::infrastructure::repositories::sqlite::sqlite_rows::cell_row::{
    cell_type_sqlite_impls::CellTypeSqlite, state_sqlite_impls::StateSqlite,
};

#[derive(Debug)]
/// Used to select cells with left join on repetitions.
pub struct CellRow {
    // Cell fields
    pub cell_id: Guid,
    pub cell_created_date: DateTime<Utc>,
    pub cell_modified_date: DateTime<Utc>,
    pub cell_file_id: Guid,
    pub cell_content: String,
    pub cell_type: CellTypeSqlite,
    pub cell_index: u32,
    pub cell_searchable_content: String,

    // Repetition fields
    pub repetition_id: Option<Guid>,
    pub repetition_created_date: Option<DateTime<Utc>>,
    pub repetition_modified_date: Option<DateTime<Utc>>,
    pub repetition_file_id: Option<Guid>,
    pub repetition_cell_id: Option<Guid>,
    pub repetition_due: Option<DateTime<Utc>>,
    pub repetition_stability: Option<f64>,
    pub repetition_difficulty: Option<f64>,
    pub repetition_elapsed_days: Option<i64>,
    pub repetition_scheduled_days: Option<i64>,
    pub repetition_reps: Option<i64>,
    pub repetition_lapses: Option<i64>,
    pub repetition_state: Option<StateSqlite>,
    pub repetition_last_review: Option<DateTime<Utc>>,
    pub repetition_additional_content: Option<String>,
}

pub fn convert_rows_to_cells(rows: Vec<CellRow>) -> Vec<Cell> {
    let mut cells_repetitions: HashMap<Guid, Vec<Repetition>> = HashMap::new();

    for row in &rows {
        if row.repetition_id.is_none() {
            continue;
        }

        let repetition = Repetition::new_unchecked(
            row.repetition_id.unwrap(),
            row.repetition_created_date.unwrap(),
            row.repetition_modified_date.unwrap(),
            row.repetition_file_id.unwrap(),
            row.repetition_cell_id.unwrap(),
            row.repetition_due.unwrap(),
            row.repetition_stability.unwrap(),
            row.repetition_difficulty.unwrap(),
            row.repetition_elapsed_days.unwrap(),
            row.repetition_scheduled_days.unwrap(),
            row.repetition_reps.unwrap(),
            row.repetition_lapses.unwrap(),
            row.repetition_state.clone().unwrap().into(),
            row.repetition_last_review,
            row.repetition_additional_content.clone(),
        );

        cells_repetitions
            .entry(row.cell_id)
            .or_default()
            .push(repetition);
    }

    let mut added_cells: HashSet<Guid> = HashSet::new();
    let mut result = Vec::new();

    for row in rows {
        if added_cells.insert(row.cell_id) {
            let cell = Cell::new_unchecked(
                row.cell_id,
                row.cell_created_date,
                row.cell_modified_date,
                row.cell_file_id,
                row.cell_content,
                row.cell_type.into(),
                row.cell_index,
                row.cell_searchable_content,
                cells_repetitions.remove(&row.cell_id).unwrap_or_default(),
            );
            result.push(cell);
        }
    }

    result
}

// TODO: move these
pub mod cell_type_sqlite_impls {
    use super::*;
    use sqlx::Sqlite;

    // Newtype wrapper — local to THIS crate, so orphan rule is satisfied
    #[derive(Clone, Debug)]
    pub struct CellTypeSqlite(pub CellType);

    // Conversion helpers
    impl From<CellType> for CellTypeSqlite {
        fn from(ct: CellType) -> Self {
            Self(ct)
        }
    }
    impl From<CellTypeSqlite> for CellType {
        fn from(ct: CellTypeSqlite) -> Self {
            ct.0
        }
    }

    impl sqlx::Type<Sqlite> for CellTypeSqlite {
        fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
            <str as sqlx::Type<Sqlite>>::type_info()
        }
    }

    impl<'r> sqlx::Decode<'r, Sqlite> for CellTypeSqlite {
        fn decode(
            value: <Sqlite as sqlx::Database>::ValueRef<'r>,
        ) -> Result<Self, sqlx::error::BoxDynError> {
            let s = <&'r str as sqlx::Decode<'r, Sqlite>>::decode(value)?;
            serde_json::from_str(s)
                .map(CellTypeSqlite)
                .map_err(|_| format!("invalid value {:?} for CellType", s).into())
        }
    }

    impl<'q> sqlx::Encode<'q, Sqlite> for CellTypeSqlite {
        fn encode_by_ref(
            &self,
            buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer<'q>,
        ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
            let val = serde_json::to_string(&self.0).expect("Cannot serialize CellType");
            <String as sqlx::Encode<'q, Sqlite>>::encode(val, buf)
        }
    }
}

pub mod state_sqlite_impls {
    use super::*;
    use sqlx::Sqlite;

    #[derive(Clone, Debug)]
    pub struct StateSqlite(pub State);

    impl From<State> for StateSqlite {
        fn from(s: State) -> Self {
            Self(s)
        }
    }
    impl From<StateSqlite> for State {
        fn from(s: StateSqlite) -> Self {
            s.0
        }
    }

    impl sqlx::Type<Sqlite> for StateSqlite {
        fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
            <str as sqlx::Type<Sqlite>>::type_info()
        }
    }

    impl<'r> sqlx::Decode<'r, Sqlite> for StateSqlite {
        fn decode(
            value: <Sqlite as sqlx::Database>::ValueRef<'r>,
        ) -> Result<Self, sqlx::error::BoxDynError> {
            let s = <&'r str as sqlx::Decode<'r, Sqlite>>::decode(value)?;
            serde_json::from_str(s)
                .map(StateSqlite)
                .map_err(|_| format!("invalid value {:?} for State", s).into())
        }
    }

    impl<'q> sqlx::Encode<'q, Sqlite> for StateSqlite {
        fn encode_by_ref(
            &self,
            buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer<'q>,
        ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
            let val = serde_json::to_string(&self.0).expect("Cannot serialize State");
            <String as sqlx::Encode<'q, Sqlite>>::encode(val, buf)
        }
    }
}

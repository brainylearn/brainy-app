use chrono::{DateTime, Utc};

use brainy_domain::{Guid, cells::entities::review::Review};

use crate::cells::review_row::rating_sqlite_impls::RatingSqlite;

pub struct ReviewRow {
    pub id: Guid,
    pub created_date: DateTime<Utc>,
    pub modified_date: DateTime<Utc>,
    pub cell_id: Option<Guid>,
    pub study_time: u32,
    pub date: DateTime<Utc>,
    pub rating: RatingSqlite,
}

impl From<ReviewRow> for Review {
    fn from(value: ReviewRow) -> Self {
        Review::new_unchecked(
            value.id,
            value.created_date,
            value.modified_date,
            value.cell_id,
            value.study_time,
            value.date,
            value.rating.into(),
        )
    }
}

// TODO: maybe use the micro provided by claude
pub mod rating_sqlite_impls {
    use brainy_domain::cells::entities::review::Rating;
    use sqlx::Sqlite;

    #[derive(Clone, Debug)]
    pub struct RatingSqlite(pub Rating);

    impl From<Rating> for RatingSqlite {
        fn from(r: Rating) -> Self {
            Self(r)
        }
    }
    impl From<RatingSqlite> for Rating {
        fn from(r: RatingSqlite) -> Self {
            r.0
        }
    }

    impl sqlx::Type<Sqlite> for RatingSqlite {
        fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
            <str as sqlx::Type<Sqlite>>::type_info()
        }
    }

    impl<'r> sqlx::Decode<'r, Sqlite> for RatingSqlite {
        fn decode(
            value: <Sqlite as sqlx::Database>::ValueRef<'r>,
        ) -> Result<Self, sqlx::error::BoxDynError> {
            let s = <&'r str as sqlx::Decode<'r, Sqlite>>::decode(value)?;
            serde_json::from_str(s)
                .map(RatingSqlite)
                .map_err(|_| format!("invalid value {:?} for Rating", s).into())
        }
    }

    impl<'q> sqlx::Encode<'q, Sqlite> for RatingSqlite {
        fn encode_by_ref(
            &self,
            buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer<'q>,
        ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
            let val = serde_json::to_string(&self.0).expect("Cannot serialize Rating");
            <String as sqlx::Encode<'q, Sqlite>>::encode(val, buf)
        }
    }
}

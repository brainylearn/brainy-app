use chrono::{DateTime, Utc};
use prost_types::Timestamp;

pub trait IntoDateTime {
    fn into_datetime(self) -> DateTime<Utc>;
}

impl IntoDateTime for Timestamp {
    fn into_datetime(self) -> DateTime<Utc> {
        let nanos = if self.nanos < 0 { 0 } else { self.nanos as u32 };
        DateTime::<Utc>::from_timestamp(self.seconds, nanos).expect("Failed to convert timestamp")
    }
}

pub trait IntoDateTimeOption {
    fn into_datetime(self) -> Option<DateTime<Utc>>;
}

impl IntoDateTimeOption for Option<Timestamp> {
    fn into_datetime(self) -> Option<DateTime<Utc>> {
        self.map(|v| v.into_datetime())
    }
}

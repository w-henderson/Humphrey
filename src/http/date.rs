use chrono::{Datelike, Timelike, Utc};

/// Represents a date in the correct format for the `Date` header.
/// The only method that should be used is `HTTPDate::now()`.
pub struct HTTPDate;

static MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

impl HTTPDate {
    pub fn now() -> String {
        let now = Utc::now();

        format!(
            "{:?}, {:02} {:02} {} {:02}:{:02}:{:02} GMT",
            now.weekday(),
            now.day(),
            MONTHS[now.month0() as usize],
            now.year(),
            now.hour(),
            now.minute(),
            now.second()
        )
    }
}

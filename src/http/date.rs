use chrono::{Datelike, Timelike, Utc};

pub struct HTTPDate;

static MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

impl HTTPDate {
    pub fn now() -> String {
        let now = Utc::now();

        format!(
            "{:?}, {} {} {} {}:{}:{} GMT",
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

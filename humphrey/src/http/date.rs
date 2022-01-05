//! Provides functionality for handling HTTP date timestamps.

use std::time::SystemTime;

const DAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const DAYS_IN_MONTHS: [i64; 12] = [31, 30, 31, 30, 31, 31, 30, 31, 30, 31, 31, 29]; // starts with March

const MINUTE: i64 = 60;
const HOUR: i64 = MINUTE * 60;
const DAY: i64 = HOUR * 24;

const DAYS_4_YEARS: i64 = 365 * 4 + 1;
const DAYS_100_YEARS: i64 = 365 * 100 + 24;
const DAYS_400_YEARS: i64 = 365 * 400 + 97;
const MARCH_01_2000: i64 = 951868800;

/// Represents a date and time.
pub struct DateTime {
    /// The UNIX timestamp of the date.
    pub timestamp: i64,
    /// The year of the date.
    pub year: u16,
    /// The month of the date.
    pub month: u8,
    /// The day of the month of the date.
    pub day: u8,
    /// The weekday of the date.
    pub weekday: u8,
    /// The hour of the date.
    pub hour: u8,
    /// The minute of the date.
    pub minute: u8,
    /// The second of the date.
    pub second: u8,
}

impl DateTime {
    /// Creates a new `DateTime` from the current time.
    pub fn now() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Self::from(timestamp)
    }

    /// Returns the UNIX timestamp of the date.
    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }
}

impl From<i64> for DateTime {
    /// Converts from a timestamp into a date and time.
    /// Implementation modified from [here](http://git.musl-libc.org/cgit/musl/tree/src/time/__secs_to_tm.c?h=v0.9.15).
    fn from(timestamp: i64) -> Self {
        let seconds = timestamp - MARCH_01_2000;
        let mut days = seconds / DAY;
        let mut remaining_seconds = seconds % 86400;
        if remaining_seconds < 0 {
            remaining_seconds += 86400;
            days -= 1;
        }

        let mut weekday = (days + 3) % 7;
        if weekday < 0 {
            weekday += 7;
        }

        let mut y400_cycles = days / DAYS_400_YEARS;
        let mut remaining_days = days % DAYS_400_YEARS;
        if remaining_days < 0 {
            remaining_days += DAYS_400_YEARS;
            y400_cycles -= 1;
        }

        let mut y100_cycles = remaining_days / DAYS_100_YEARS;
        if y100_cycles == 4 {
            y100_cycles -= 1;
        }
        remaining_days -= y100_cycles * DAYS_100_YEARS;

        let mut y4_cycles = remaining_days / DAYS_4_YEARS;
        if y4_cycles == 25 {
            y4_cycles -= 1;
        }
        remaining_days -= y4_cycles * DAYS_4_YEARS;

        let mut remaining_years = remaining_days / 365;
        if remaining_years == 4 {
            remaining_years -= 1;
        }
        remaining_days -= remaining_years * 365;

        let mut year =
            (remaining_years + 4 * y4_cycles + 100 * y100_cycles + 400 * y400_cycles) + 2000;

        let mut months = 0;
        while DAYS_IN_MONTHS[months] <= remaining_days {
            remaining_days -= DAYS_IN_MONTHS[months];
            months += 1;
        }

        let mut month = months + 2;
        if month >= 12 {
            month -= 12;
            year += 1;
        }

        let day = remaining_days + 1;
        let hour = remaining_seconds / 3600;
        let minute = remaining_seconds / 60 % 60;
        let second = remaining_seconds % 60;

        Self {
            timestamp,
            year: year as u16,
            month: month as u8,
            day: day as u8,
            weekday: weekday as u8,
            hour: hour as u8,
            minute: minute as u8,
            second: second as u8,
        }
    }
}

impl ToString for DateTime {
    /// Returns a string formatted as an HTTP date representing the `DateTime`.
    ///
    /// ## Example
    /// ```
    /// let date = DateTime::now();
    /// println!("{}", date.to_string()); // "Thu, 01 Jan 1970 00:00:00 GMT"
    /// ```
    fn to_string(&self) -> String {
        format!(
            "{}, {:02} {:02} {} {:02}:{:02}:{:02} GMT",
            DAYS[self.weekday as usize],
            self.day,
            MONTHS[self.month as usize],
            self.year,
            self.hour,
            self.minute,
            self.second
        )
    }
}

use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SECONDS_PER_MINUTE: u64 = 60;
const SECONDS_PER_HOUR: u64 = 60 * SECONDS_PER_MINUTE;
const SECONDS_PER_DAY: u64 = 24 * SECONDS_PER_HOUR;

pub(crate) fn current_unix_timestamp() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
}

pub(crate) fn format_utc_timestamp(timestamp: Duration) -> String {
    let total_seconds = timestamp.as_secs();
    let days_since_unix_epoch = (total_seconds / SECONDS_PER_DAY) as i64;
    let seconds_since_midnight = total_seconds % SECONDS_PER_DAY;
    let (year, month, day) = civil_from_days(days_since_unix_epoch);
    let hour = seconds_since_midnight / SECONDS_PER_HOUR;
    let minute = (seconds_since_midnight % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE;
    let second = seconds_since_midnight % SECONDS_PER_MINUTE;

    format!("{year:04}-{month:02}-{day:02}_{hour:02}-{minute:02}-{second:02}")
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i32, u8, u8) {
    let shifted_days = days_since_unix_epoch + 719_468;
    let era = if shifted_days >= 0 {
        shifted_days
    } else {
        shifted_days - 146_096
    } / 146_097;
    let day_of_era = shifted_days - (era * 146_097);
    let year_of_era =
        (day_of_era - (day_of_era / 1_460) + (day_of_era / 36_524) - (day_of_era / 146_096)) / 365;
    let mut year = year_of_era + (era * 400);
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_piece = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_piece + 2) / 5 + 1;
    let month = month_piece + if month_piece < 10 { 3 } else { -9 };
    if month <= 2 {
        year += 1;
    }

    (year as i32, month as u8, day as u8)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::format_utc_timestamp;

    #[test]
    fn format_utc_timestamp_uses_calendar_date_and_time() {
        let timestamp = format_utc_timestamp(Duration::new(1_741_018_296, 0));

        assert_eq!(timestamp, "2025-03-03_16-11-36");
    }
}

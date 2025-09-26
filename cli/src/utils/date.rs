use chrono::{DateTime, NaiveDate, Utc, Datelike, Timelike};

/// Parse Vietnam date string (YYYY-MM-DD) to UTC DateTime
pub fn parse_vietnam_date(date_str: &str) -> anyhow::Result<DateTime<Utc>> {
    let naive_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
    let datetime = naive_date
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| anyhow::anyhow!("Invalid time components"))?;
    Ok(datetime.and_utc())
}

/// Format DateTime to Vietnam date string (YYYY-MM-DD)
pub fn format_vietnam_date(date: DateTime<Utc>) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// Check if date is after or equal to another date
pub fn is_date_after_or_equal(date: DateTime<Utc>, other: DateTime<Utc>) -> bool {
    date.date_naive() >= other.date_naive()
}

/// Check if date is before or equal to another date
pub fn is_date_before_or_equal(date: DateTime<Utc>, other: DateTime<Utc>) -> bool {
    date.date_naive() <= other.date_naive()
}

/// Get today's date in Vietnam timezone as string
pub fn get_today_vietnam() -> String {
    let now = Utc::now();
    format_vietnam_date(now)
}

/// Check if a date string represents today
pub fn is_today(date_str: &str) -> bool {
    date_str == get_today_vietnam()
}

/// Get yesterday's date as string
pub fn get_yesterday_vietnam() -> String {
    let yesterday = Utc::now() - chrono::Duration::days(1);
    format_vietnam_date(yesterday)
}

/// Check if it's weekend (Saturday or Sunday)
pub fn is_weekend(date: DateTime<Utc>) -> bool {
    let weekday = date.date_naive().weekday();
    weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun
}

/// Get the latest trading day (accounting for weekends)
pub fn get_latest_trading_day() -> DateTime<Utc> {
    let today = Utc::now();
    let weekday = today.date_naive().weekday();

    match weekday {
        chrono::Weekday::Sun => today - chrono::Duration::days(2), // Sunday -> Friday
        chrono::Weekday::Sat => today - chrono::Duration::days(1), // Saturday -> Friday
        _ => today, // Weekdays
    }
}

/// Check if live data is stale (older than latest trading day)
pub fn is_live_data_stale(data_date: DateTime<Utc>) -> bool {
    let latest_trading_day = get_latest_trading_day();
    let latest_trading_date = latest_trading_day.date_naive();
    let data_date_only = data_date.date_naive();

    // Data is stale if it's older than the latest trading day
    data_date_only < latest_trading_date
}

/// Generate a list of trading days between two dates (excluding weekends)
pub fn get_trading_days_between(
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> Vec<String> {
    let mut trading_days = Vec::new();
    let mut current_date = start_date;

    while current_date <= end_date {
        if !is_weekend(current_date) {
            trading_days.push(format_vietnam_date(current_date));
        }
        current_date += chrono::Duration::days(1);
    }

    trading_days
}

/// Get the number of trading days in a time range (approximately)
pub fn get_approximate_trading_days(time_range: &crate::models::TimeRange) -> usize {
    match time_range {
        crate::models::TimeRange::OneWeek => 5,
        crate::models::TimeRange::TwoWeeks => 10,
        crate::models::TimeRange::OneMonth => 22,
        crate::models::TimeRange::TwoMonths => 44,
        crate::models::TimeRange::ThreeMonths => 66,
        crate::models::TimeRange::FourMonths => 88,
        crate::models::TimeRange::SixMonths => 132,
        crate::models::TimeRange::OneYear => 252,
        crate::models::TimeRange::TwoYears => 504,
        crate::models::TimeRange::All => 2000, // Large number
        crate::models::TimeRange::Custom => 100, // Default estimate
    }
}

/// Calculate date range bounds for time ranges
pub fn calculate_date_range_bounds(
    time_range: &crate::models::TimeRange,
) -> (DateTime<Utc>, DateTime<Utc>) {
    let now = Utc::now();
    let start_date = match time_range {
        crate::models::TimeRange::OneWeek => now - chrono::Duration::weeks(1),
        crate::models::TimeRange::TwoWeeks => now - chrono::Duration::weeks(2),
        crate::models::TimeRange::OneMonth => now - chrono::Duration::weeks(4),
        crate::models::TimeRange::TwoMonths => now - chrono::Duration::weeks(8),
        crate::models::TimeRange::ThreeMonths => now - chrono::Duration::weeks(12),
        crate::models::TimeRange::FourMonths => now - chrono::Duration::weeks(16),
        crate::models::TimeRange::SixMonths => now - chrono::Duration::weeks(24),
        crate::models::TimeRange::OneYear => now - chrono::Duration::weeks(52),
        crate::models::TimeRange::TwoYears => now - chrono::Duration::weeks(104),
        crate::models::TimeRange::All => DateTime::from_timestamp(0, 0).unwrap(),
        crate::models::TimeRange::Custom => now, // Will be overridden
    };

    (start_date, now)
}

/// Check if a date falls within business hours (9:00-15:00 Vietnam time)
pub fn is_market_hours(date: DateTime<Utc>) -> bool {
    // Convert to Vietnam timezone (+7)
    let vietnam_time = date + chrono::Duration::hours(7);
    let hour = vietnam_time.hour();

    // Vietnamese stock market hours: 9:00 AM to 3:00 PM
    hour >= 9 && hour < 15
}

/// Get next market open time
pub fn get_next_market_open() -> DateTime<Utc> {
    let now = Utc::now();
    let vietnam_now = now + chrono::Duration::hours(7);

    // If it's currently market hours on a weekday, return current time
    if is_market_hours(now) && !is_weekend(now) {
        return now;
    }

    // Otherwise, find next 9 AM on a weekday
    let mut next_open = vietnam_now
        .date_naive()
        .and_hms_opt(9, 0, 0)
        .unwrap()
        .and_utc()
        - chrono::Duration::hours(7); // Convert back to UTC

    // If we've passed 9 AM today, move to tomorrow
    if vietnam_now.hour() >= 9 {
        next_open += chrono::Duration::days(1);
    }

    // Skip weekends
    while is_weekend(next_open) {
        next_open += chrono::Duration::days(1);
    }

    next_open
}

/// Format duration for logging
pub fn format_duration(duration_ms: f64) -> String {
    if duration_ms < 1000.0 {
        format!("{:.1}ms", duration_ms)
    } else if duration_ms < 60000.0 {
        format!("{:.1}s", duration_ms / 1000.0)
    } else {
        let minutes = (duration_ms / 60000.0).floor();
        let seconds = (duration_ms % 60000.0) / 1000.0;
        format!("{}m{:.1}s", minutes, seconds)
    }
}
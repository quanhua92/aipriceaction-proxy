use crate::models::StockDataPoint;

/// Clean HTML text by removing HTML tags and normalizing whitespace
pub fn clean_html_text(text: &str) -> String {
    // Simple HTML tag removal (for more complex HTML, consider using a proper HTML parser)
    let mut result = text
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&nbsp;", " ");

    // Remove HTML tags using regex-like approach
    while let Some(start) = result.find('<') {
        if let Some(end) = result[start..].find('>') {
            result.replace_range(start..(start + end + 1), "");
        } else {
            break;
        }
    }

    // Normalize whitespace
    result
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

/// Find ticker sector from ticker groups
pub fn find_ticker_sector(ticker_groups: &crate::models::TickerGroups, ticker: &str) -> Option<String> {
    ticker_groups.find_ticker_sector(ticker)
}

/// Get all tickers from ticker groups
pub fn get_all_tickers(ticker_groups: &crate::models::TickerGroups) -> Vec<String> {
    ticker_groups.get_all_tickers()
}

/// Calculate daily change for stock data point
pub fn calculate_daily_change(current: &StockDataPoint, previous: &StockDataPoint) -> (f64, f64) {
    let change = current.close - previous.close;
    let change_percent = (change / previous.close) * 100.0;
    (change, change_percent)
}

/// Format number with thousands separator
pub fn format_number_with_separator(value: f64) -> String {
    if value >= 1_000_000.0 {
        format!("{:.1}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("{:.0}k", value / 1_000.0)
    } else {
        format!("{:.2}", value)
    }
}

/// Format volume with appropriate suffix
pub fn format_volume(volume: f64) -> String {
    if volume >= 1_000_000.0 {
        format!("{:.1}M", volume / 1_000_000.0)
    } else if volume >= 1_000.0 {
        format!("{:.0}k", volume / 1_000.0)
    } else {
        format!("{:.0}", volume)
    }
}

/// Format percentage with sign
pub fn format_percentage_with_sign(value: f64) -> String {
    if value > 0.0 {
        format!("+{:.2}%", value)
    } else {
        format!("{:.2}%", value)
    }
}

/// Filter data by context date
pub fn filter_data_by_context_date(
    data: Vec<StockDataPoint>,
    context_date: Option<&str>,
) -> Vec<StockDataPoint> {
    if let Some(date) = context_date {
        data.into_iter()
            .filter(|point| point.date.to_string().split('T').next().unwrap_or("") <= date)
            .collect()
    } else {
        data
    }
}
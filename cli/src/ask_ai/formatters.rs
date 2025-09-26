use crate::models::StockDataPoint;
use crate::utils::money_flow_utils::MoneyFlowTickerData;
use super::types::{TickerAIData, MAPeriod};
use super::utils::{calculate_daily_change, format_number_with_separator, format_volume, format_percentage_with_sign, filter_data_by_context_date};

/// Format chart data to context string
pub fn format_chart_context(
    ticker: &str,
    data: Vec<StockDataPoint>,
    max_days: usize,
    context_date: Option<&str>,
) -> String {
    if data.is_empty() || max_days == 0 {
        return if max_days == 0 {
            String::new()
        } else {
            format!("{}: No chart data available", ticker)
        };
    }

    // Filter data by context date if provided
    let filtered_data = filter_data_by_context_date(data, context_date);

    // Sort data chronologically and get the last N trading days
    let mut sorted_data = filtered_data;
    sorted_data.sort_by(|a, b| a.date.cmp(&b.date));
    let recent_data: Vec<_> = sorted_data.iter().rev().take(max_days).rev().collect();

    let context_lines: Vec<String> = recent_data
        .iter()
        .enumerate()
        .map(|(index, point)| {
            let date_str = point.date.format("%Y-%m-%d").to_string();

            // Calculate daily change if not the first point
            let change_str = if index > 0 {
                let (change, change_percent) = calculate_daily_change(point, recent_data[index - 1]);
                format!(", Change={} ({})",
                    format_percentage_with_sign(change),
                    format_percentage_with_sign(change_percent)
                )
            } else {
                String::new()
            };

            format!(
                "{}: Date={}, Open={}, High={}, Low={}, Close={}, Volume={}{}",
                ticker,
                date_str,
                format_number_with_separator(point.open),
                format_number_with_separator(point.high),
                format_number_with_separator(point.low),
                format_number_with_separator(point.close),
                format_volume(point.volume as f64),
                change_str
            )
        })
        .collect();

    format!("# Last {} Trading Days OHLCV Data\n{}", max_days, context_lines.join("\n"))
}

/// Format VPA data to context string
pub fn format_vpa_context(
    ticker: &str,
    vpa_content: Option<&str>,
    max_days: usize,
    context_date: Option<&str>,
) -> String {
    let vpa_content = match vpa_content {
        Some(content) if max_days > 0 => content,
        _ => return if max_days == 0 {
            String::new()
        } else {
            format!("{} VPA: No VPA data available", ticker)
        }
    };

    // Extract last N rows of meaningful VPA data
    let lines: Vec<&str> = vpa_content.split('\n').filter(|line| !line.trim().is_empty()).collect();

    // Find lines that look like data rows (contain pipes, dates, or structured content)
    let mut data_lines: Vec<&str> = lines
        .iter()
        .filter(|line| {
            line.contains('|') ||
            (line.contains("Date") && line.contains("Action")) ||
            line.contains("VNINDEX") ||
            line.contains("Ngày") ||
            regex::Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap().is_match(line)
        })
        .copied()
        .collect();

    // Filter by context date if provided
    if let Some(context_date) = context_date {
        data_lines = data_lines
            .into_iter()
            .filter(|line| {
                // Try multiple date patterns
                let date_patterns = [
                    regex::Regex::new(r"(\d{4}-\d{2}-\d{2})").unwrap(),
                    regex::Regex::new(r"Ngày\s+(\d{4}-\d{2}-\d{2})").unwrap(),
                    regex::Regex::new(r"\*\*Ngày\s+(\d{4}-\d{2}-\d{2})").unwrap(),
                ];

                for pattern in &date_patterns {
                    if let Some(captures) = pattern.captures(line) {
                        if let Some(date_match) = captures.get(1) {
                            return date_match.as_str() <= context_date;
                        }
                    }
                }

                // Keep lines without dates
                !regex::Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap().is_match(line)
            })
            .collect();
    }

    // Get last N relevant lines
    let recent_vpa: Vec<&str> = data_lines.iter().rev().take(max_days).rev().copied().collect();

    if recent_vpa.is_empty() {
        return format!(
            "{} VPA: No VPA data available{}",
            ticker,
            if context_date.is_some() {
                format!(" for dates on or before {}", context_date.unwrap())
            } else {
                String::new()
            }
        );
    }

    // Add TICKER: prefix to each VPA data line
    let prefixed_vpa: Vec<String> = recent_vpa
        .iter()
        .map(|line| format!("{}: {}", ticker, line))
        .collect();

    format!("{} VPA:\n{}", ticker, prefixed_vpa.join("\n"))
}

/// Format MA Score data to context string
pub fn format_ma_score_context(
    ticker: &str,
    ma_score_data: Option<&std::collections::HashMap<String, Vec<crate::models::ma_score::MAScoreTickerData>>>,
    max_days: usize,
    ma_period: MAPeriod,
    context_date: Option<&str>,
) -> String {
    let ma_score_data = match ma_score_data {
        Some(data) if max_days > 0 => data,
        _ => return if max_days == 0 {
            String::new()
        } else {
            format!("{}: No MA Score data available", ticker)
        }
    };

    // Get all available dates and filter by contextDate if provided
    let mut all_dates: Vec<String> = ma_score_data.keys().cloned().collect();
    all_dates.sort();

    let filtered_dates = if let Some(context_date) = context_date {
        all_dates.into_iter().filter(|date| date.as_str() <= context_date).collect()
    } else {
        all_dates
    };

    // Get the last N trading days
    let recent_dates: Vec<String> = filtered_dates.iter().rev().take(max_days).rev().cloned().collect();

    if recent_dates.is_empty() {
        return format!(
            "{}: No MA Score data available{}",
            ticker,
            if context_date.is_some() {
                format!(" for dates on or before {}", context_date.unwrap())
            } else {
                String::new()
            }
        );
    }

    let context_lines: Vec<String> = recent_dates
        .iter()
        .filter_map(|date| {
            let date_data = ma_score_data.get(date)?;
            let ticker_data = date_data.iter().find(|data| data.ticker == ticker)?;

            // Get the MA score for this specific date
            let ma_score = match ma_period {
                MAPeriod::MA10 => ticker_data.ma10_scores.get(date).copied(),
                MAPeriod::MA20 => ticker_data.ma20_scores.get(date).copied(),
                MAPeriod::MA50 => ticker_data.ma50_scores.get(date).copied(),
            }?;

            // Format without close and MA value since they're not in the current structure
            Some(format!(
                "{}: Date={}, MA{}Score={}% (Trend: {:.2})",
                ticker,
                date,
                ma_period as u32,
                format_percentage_with_sign(ma_score),
                ticker_data.trend_score
            ))
        })
        .collect();

    if context_lines.is_empty() {
        format!("{}: No matching MA Score data found", ticker)
    } else {
        format!("# Last {} Trading Days MA{} Score Data\n{}", max_days, ma_period as u32, context_lines.join("\n"))
    }
}

/// Format ticker AI data to context string
pub fn format_ticker_ai_context(
    ticker: &str,
    ticker_ai_data: Option<&TickerAIData>,
    include_basic_info: bool,
    include_financial_ratios: bool,
    include_description: bool,
) -> Option<String> {
    let data = ticker_ai_data?;
    let mut sections = Vec::new();

    if include_basic_info {
        let mut basic_info = Vec::new();

        if let Some(company_name) = &data.company_name {
            basic_info.push(format!("Company: {}", company_name));
        }

        if let Some(market_cap) = data.market_cap {
            basic_info.push(format!("Market Cap: {}", format_number_with_separator(market_cap)));
        }

        if !basic_info.is_empty() {
            sections.push(format!("## {} Basic Information\n{}", ticker, basic_info.join(", ")));
        }
    }

    if include_financial_ratios {
        let mut ratios = Vec::new();

        if let Some(pe) = data.pe_ratio {
            ratios.push(format!("P/E: {:.2}", pe));
        }
        if let Some(pb) = data.pb_ratio {
            ratios.push(format!("P/B: {:.2}", pb));
        }
        if let Some(roe) = data.roe {
            ratios.push(format!("ROE: {}%", format_percentage_with_sign(roe)));
        }
        if let Some(roa) = data.roa {
            ratios.push(format!("ROA: {}%", format_percentage_with_sign(roa)));
        }
        if let Some(debt_equity) = data.debt_to_equity {
            ratios.push(format!("D/E: {:.2}", debt_equity));
        }
        if let Some(current_ratio) = data.current_ratio {
            ratios.push(format!("Current Ratio: {:.2}", current_ratio));
        }

        if !ratios.is_empty() {
            sections.push(format!("## {} Financial Ratios\n{}", ticker, ratios.join(", ")));
        }
    }

    if include_description {
        if let Some(description) = &data.description {
            let clean_desc = super::utils::clean_html_text(description);
            if !clean_desc.is_empty() {
                sections.push(format!("## {} Company Description\n{}", ticker, clean_desc));
            }
        }
    }

    if sections.is_empty() {
        None
    } else {
        Some(sections.join("\n\n"))
    }
}

/// Format money flow data to context string
pub fn format_money_flow_context(
    ticker: &str,
    money_flow_data: &MoneyFlowTickerData,
    max_days: usize,
    context_date: Option<&str>,
) -> String {
    // Get available dates from signed percentage data
    let mut dates: Vec<String> = money_flow_data.signed_percentage_data.keys().cloned().collect();
    dates.sort();

    let filtered_dates = if let Some(context_date) = context_date {
        dates.into_iter().filter(|date| date.as_str() <= context_date).collect()
    } else {
        dates
    };

    let recent_dates: Vec<String> = filtered_dates.iter().rev().take(max_days).rev().cloned().collect();

    if recent_dates.is_empty() {
        return format!("{}: No money flow data available", ticker);
    }

    let flow_timeline: Vec<String> = recent_dates
        .iter()
        .filter_map(|date| {
            let signed_flow = money_flow_data.signed_percentage_data.get(date).copied().unwrap_or(0.0);
            let volume_data = money_flow_data.volume_data.get(date)?;

            Some(format!(
                "{}: Date={}, MoneyFlow={}%, Volume={}, VolumeChange={}%",
                ticker,
                date,
                format_percentage_with_sign(signed_flow),
                format_volume(volume_data.volume),
                format_percentage_with_sign(volume_data.change)
            ))
        })
        .collect();

    if flow_timeline.is_empty() {
        format!("{}: No money flow timeline data available", ticker)
    } else {
        format!(
            "# Last {} Trading Days Money Flow Data\n{}: TrendScore={:.2}, MarketCap={}\n{}",
            max_days,
            ticker,
            money_flow_data.trend_score,
            format_number_with_separator(money_flow_data.market_cap),
            flow_timeline.join("\n")
        )
    }
}
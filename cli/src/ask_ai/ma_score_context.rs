use std::collections::HashMap;
use crate::models::StockDataPoint;
use crate::models::ma_score::MAScoreTickerData;
use super::context_builders::build_vnindex_market_context;
use super::types::{ContextConfig, MAPeriod};
use super::utils::{find_ticker_sector, format_percentage_with_sign};

/// Build market-wide MA Score context for sector rotation and momentum analysis
pub fn build_market_ma_score_context(
    _all_ticker_data: &HashMap<String, Vec<StockDataPoint>>,
    ticker_groups: &crate::models::TickerGroups,
    vnindex_data: &[StockDataPoint],
    ma_score_data: Option<&HashMap<String, Vec<MAScoreTickerData>>>,
    config: &ContextConfig,
) -> String {
    let mut context_lines = Vec::new();

    // 1. VNINDEX Chart Context
    let vnindex_context = build_vnindex_market_context(vnindex_data, config);
    if !vnindex_context.is_empty() {
        context_lines.push(vnindex_context);
    }

    // 2. MA Score analysis if available
    if let Some(ma_data) = ma_score_data {
        // Top MA Score performers
        let ma_leaders = build_ma_score_leaders_context(ma_data, config);
        if !ma_leaders.is_empty() {
            context_lines.push(ma_leaders);
        }

        // Sector MA Score summary
        let sector_ma_summary = build_sector_ma_score_summary(ma_data, ticker_groups, config);
        if !sector_ma_summary.is_empty() {
            context_lines.push(sector_ma_summary);
        }

        // MA Score momentum analysis
        let momentum_analysis = build_ma_score_momentum_analysis(ma_data, config);
        if !momentum_analysis.is_empty() {
            context_lines.push(momentum_analysis);
        }
    }

    context_lines.join("\n\n")
}

/// Get MA score for a ticker on a specific date based on configured period
fn get_ma_score_for_date(data: &MAScoreTickerData, date: &str, ma_period: MAPeriod) -> Option<f64> {
    match ma_period {
        MAPeriod::MA10 => data.ma10_scores.get(date).copied(),
        MAPeriod::MA20 => data.ma20_scores.get(date).copied(),
        MAPeriod::MA50 => data.ma50_scores.get(date).copied(),
    }
}

/// Build MA Score leaders context showing top performers
fn build_ma_score_leaders_context(
    ma_score_data: &HashMap<String, Vec<MAScoreTickerData>>,
    config: &ContextConfig,
) -> String {
    if ma_score_data.is_empty() {
        return String::new();
    }

    // Get the most recent date
    let mut dates: Vec<String> = ma_score_data.keys().cloned().collect();
    dates.sort();

    let recent_date = if let Some(context_date) = &config.context_date {
        dates.into_iter()
            .filter(|date| date <= context_date)
            .last()
    } else {
        dates.last().cloned()
    };

    let recent_date = match recent_date {
        Some(date) => date,
        None => return String::new(),
    };

    // Get recent data
    let recent_data = match ma_score_data.get(&recent_date) {
        Some(data) => data,
        None => return String::new(),
    };

    // Collect scores for the recent date and sort by MA score
    let mut ticker_scores: Vec<(String, f64)> = recent_data
        .iter()
        .filter(|data| data.ticker != "VNINDEX")
        .filter_map(|data| {
            get_ma_score_for_date(data, &recent_date, config.ma_period)
                .map(|score| (data.ticker.clone(), score))
        })
        .collect();

    // Sort by score (descending)
    ticker_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Get top 10 performers
    let top_performers: Vec<String> = ticker_scores
        .iter()
        .take(10)
        .map(|(ticker, score)| {
            // Find ticker data for additional info
            if let Some(ticker_data) = recent_data.iter().find(|d| d.ticker == *ticker) {
                format!(
                    "{}: MA{} Score={}% (Trend Score: {:.2})",
                    ticker,
                    config.ma_period as u32,
                    format_percentage_with_sign(*score),
                    ticker_data.trend_score
                )
            } else {
                format!(
                    "{}: MA{} Score={}%",
                    ticker,
                    config.ma_period as u32,
                    format_percentage_with_sign(*score)
                )
            }
        })
        .collect();

    if top_performers.is_empty() {
        String::new()
    } else {
        format!(
            "# Top 10 MA{} Score Leaders (Date: {})\n{}",
            config.ma_period as u32,
            recent_date,
            top_performers.join("\n")
        )
    }
}

/// Build sector MA Score summary
fn build_sector_ma_score_summary(
    ma_score_data: &HashMap<String, Vec<MAScoreTickerData>>,
    ticker_groups: &crate::models::TickerGroups,
    config: &ContextConfig,
) -> String {
    if ma_score_data.is_empty() {
        return String::new();
    }

    // Get the most recent date
    let mut dates: Vec<String> = ma_score_data.keys().cloned().collect();
    dates.sort();

    let recent_date = if let Some(context_date) = &config.context_date {
        dates.into_iter()
            .filter(|date| date <= context_date)
            .last()
    } else {
        dates.last().cloned()
    };

    let recent_date = match recent_date {
        Some(date) => date,
        None => return String::new(),
    };

    let recent_data = match ma_score_data.get(&recent_date) {
        Some(data) => data,
        None => return String::new(),
    };

    // Group by sector
    let mut sector_scores: HashMap<String, Vec<f64>> = HashMap::new();
    for data in recent_data {
        if data.ticker == "VNINDEX" {
            continue;
        }

        let sector = find_ticker_sector(ticker_groups, &data.ticker)
            .unwrap_or("Unknown".to_string());

        if let Some(ma_score) = get_ma_score_for_date(data, &recent_date, config.ma_period) {
            sector_scores.entry(sector).or_default().push(ma_score);
        }
    }

    // Calculate sector averages
    let mut sector_summaries: Vec<(String, f64, usize)> = sector_scores
        .iter()
        .map(|(sector, scores)| {
            let avg_score = scores.iter().sum::<f64>() / scores.len() as f64;
            (sector.clone(), avg_score, scores.len())
        })
        .collect();

    // Sort by average MA score
    sector_summaries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    if sector_summaries.is_empty() {
        return String::new();
    }

    let sector_lines: Vec<String> = sector_summaries
        .iter()
        .take(10)
        .map(|(sector, avg_score, count)| {
            format!(
                "- {} ({} stocks): AvgScore={}%",
                sector,
                count,
                format_percentage_with_sign(*avg_score)
            )
        })
        .collect();

    format!(
        "# Sector MA{} Score Analysis (Date: {})\n{}",
        config.ma_period as u32,
        recent_date,
        sector_lines.join("\n")
    )
}

/// Build MA Score momentum analysis
fn build_ma_score_momentum_analysis(
    ma_score_data: &HashMap<String, Vec<MAScoreTickerData>>,
    config: &ContextConfig,
) -> String {
    if ma_score_data.is_empty() {
        return String::new();
    }

    // Get recent dates for momentum comparison
    let mut dates: Vec<String> = ma_score_data.keys().cloned().collect();
    dates.sort();

    let filtered_dates = if let Some(context_date) = &config.context_date {
        dates.into_iter()
            .filter(|date| date <= context_date)
            .collect()
    } else {
        dates
    };

    if filtered_dates.len() < 2 {
        return String::new();
    }

    // Get last 2 dates for momentum calculation
    let recent_dates: Vec<String> = filtered_dates
        .iter()
        .rev()
        .take(2)
        .rev()
        .cloned()
        .collect();

    if recent_dates.len() < 2 {
        return String::new();
    }

    let previous_date = &recent_dates[0];
    let current_date = &recent_dates[1];

    let previous_data = ma_score_data.get(previous_date);
    let current_data = ma_score_data.get(current_date);

    if previous_data.is_none() || current_data.is_none() {
        return String::new();
    }

    let previous_data = previous_data.unwrap();
    let current_data = current_data.unwrap();

    // Calculate momentum for each ticker
    let mut momentum_data = Vec::new();
    for current_ticker in current_data {
        if current_ticker.ticker == "VNINDEX" {
            continue;
        }

        if let Some(previous_ticker) = previous_data.iter().find(|t| t.ticker == current_ticker.ticker) {
            let current_score = get_ma_score_for_date(current_ticker, current_date, config.ma_period);
            let previous_score = get_ma_score_for_date(previous_ticker, previous_date, config.ma_period);

            if let (Some(current), Some(previous)) = (current_score, previous_score) {
                let momentum = current - previous;
                momentum_data.push((current_ticker.ticker.clone(), current, momentum));
            }
        }
    }

    // Sort by momentum
    momentum_data.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    // Get top gainers and losers
    let top_gainers: Vec<String> = momentum_data
        .iter()
        .take(5)
        .map(|(ticker, current_score, momentum)| {
            format!(
                "{}: Score={}% ({}{})",
                ticker,
                format_percentage_with_sign(*current_score),
                if *momentum > 0.0 { "+" } else { "" },
                format!("{:.2}%", momentum)
            )
        })
        .collect();

    let top_losers: Vec<String> = momentum_data
        .iter()
        .rev()
        .take(5)
        .map(|(ticker, current_score, momentum)| {
            format!(
                "{}: Score={}% ({}{})",
                ticker,
                format_percentage_with_sign(*current_score),
                if *momentum > 0.0 { "+" } else { "" },
                format!("{:.2}%", momentum)
            )
        })
        .collect();

    if top_gainers.is_empty() && top_losers.is_empty() {
        String::new()
    } else {
        format!(
            "# MA{} Score Momentum Analysis ({} → {})\n## Top Momentum Gainers\n{}\n\n## Top Momentum Losers\n{}",
            config.ma_period as u32,
            previous_date,
            current_date,
            top_gainers.join("\n"),
            top_losers.join("\n")
        )
    }
}
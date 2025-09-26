use crate::ask_ai::{
    types::{Language, ContextConfig, MAPeriod},
    templates::get_template_by_id,
};
use crate::services::CacheManager;

pub async fn handle_ask_ai_request(
    tickers: String,
    template_id: String,
    language: String,
    chart_days: usize,
    money_flow_days: usize,
    ma_score_days: usize,
    ma_period: u32,
    context_date: Option<String>,
) -> anyhow::Result<()> {
    println!("🤖 Generating AI analysis prompt...");

    // Parse language
    let lang = match language.to_lowercase().as_str() {
        "vn" | "vi" | "vietnamese" => Language::Vietnamese,
        _ => Language::English,
    };

    // Parse MA period
    let ma_period_enum = match ma_period {
        10 => MAPeriod::MA10,
        20 => MAPeriod::MA20,
        50 => MAPeriod::MA50,
        _ => {
            eprintln!("❌ Invalid MA period: {}. Must be 10, 20, or 50", ma_period);
            return Ok(());
        }
    };

    // Parse tickers
    let ticker_list: Vec<String> = tickers
        .split(',')
        .map(|t| t.trim().to_uppercase())
        .filter(|t| !t.is_empty())
        .collect();

    if ticker_list.is_empty() {
        eprintln!("❌ No valid tickers provided");
        return Ok(());
    }

    // Get template
    let template = match get_template_by_id(&template_id, &lang) {
        Some(template) => template,
        None => {
            eprintln!("❌ Template '{}' not found for language {:?}", template_id, lang);
            println!("💡 Available templates will be listed soon...");
            return Ok(());
        }
    };

    // Create config
    let _config = ContextConfig {
        chart_context_days: chart_days,
        money_flow_context_days: money_flow_days,
        ma_score_context_days: ma_score_days,
        ma_period: ma_period_enum,
        context_date: context_date.clone(),
        ..Default::default()
    };

    println!("📊 Analyzing {} ticker(s): {}", ticker_list.len(), ticker_list.join(", "));
    println!("🎯 Template: {}", template.title);
    println!("🌐 Language: {:?}", lang);
    if let Some(ref date) = context_date {
        println!("📅 Context Date: {}", date);
    }

    // Initialize cache manager
    let _cache = CacheManager::new();

    // For now, we'll provide a basic structure since we don't have
    // the full data pipeline connected yet
    println!("\n{}", "=".repeat(80));
    println!("🤖 AI ANALYSIS PROMPT");
    println!("{}", "=".repeat(80));

    println!("\n📝 PROMPT TEMPLATE:");
    println!("{}", template.prompt);

    println!("\n📊 ANALYSIS CONTEXT:");
    println!("Tickers: {}", ticker_list.join(", "));
    println!("Chart Context Days: {}", chart_days);
    println!("Money Flow Context Days: {}", money_flow_days);
    println!("MA Score Context Days: {}", ma_score_days);
    println!("MA Period: MA{}", ma_period);
    if let Some(ref date) = context_date {
        println!("Context Date: {}", date);
    }

    // Note: In a full implementation, here we would:
    // 1. Fetch data from cache for each ticker
    // 2. Build the complete context using context builders
    // 3. Combine template + context for the final prompt

    println!("\n💡 NEXT STEPS:");
    println!("1. Copy the prompt template above");
    println!("2. Add relevant market data context for: {}", ticker_list.join(", "));
    println!("3. Paste into your preferred AI assistant (ChatGPT, Claude, etc.)");

    println!("\n{}", "=".repeat(80));

    Ok(())
}
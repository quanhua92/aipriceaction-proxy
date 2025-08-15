use aipriceaction_proxy::vci::{VciClient, VciError};

#[tokio::main]
async fn main() -> Result<(), VciError> {
    println!("VCI Client Example");
    println!("==================");

    let mut client = VciClient::new(true, 6)?;
    let test_symbol = "VCI";

    // 1. Test company info
    println!("\n🏢 Company Information for {}", test_symbol);
    println!("{}", "-".repeat(40));
    
    match client.company_info(test_symbol).await {
        Ok(company_data) => {
            println!("✅ Success! Company data retrieved");
            println!("📊 Exchange: {:?}", company_data.exchange);
            println!("🏭 Industry: {:?}", company_data.industry);
            
            if let Some(market_cap) = company_data.market_cap {
                let market_cap_b = market_cap / 1_000_000_000.0;
                println!("💰 Market Cap: {:.1}B VND", market_cap_b);
            }
            
            if let Some(shares) = company_data.outstanding_shares {
                println!("📈 Outstanding Shares: {}", shares);
            }
            
            println!("👥 Shareholders: {} major", company_data.shareholders.len());
            println!("👔 Officers: {} management", company_data.officers.len());
        }
        Err(e) => println!("❌ Failed to retrieve company data: {:?}", e),
    }

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // 2. Test historical data
    println!("\n📈 Historical Data for {}", test_symbol);
    println!("{}", "-".repeat(40));
    
    match client.get_history(test_symbol, "2025-08-01", Some("2025-08-15"), "1D").await {
        Ok(data) => {
            let data_count = data.len();
            println!("✅ Success! Retrieved {} data points", data_count);
            
            if !data.is_empty() {
                let first = &data[0];
                let last = &data[data.len() - 1];
                println!("📅 Range: {} to {}", first.time.format("%Y-%m-%d"), last.time.format("%Y-%m-%d"));
                println!("💹 Latest: {:.0} VND (Vol: {})", last.close, last.volume);
                
                if data.len() > 1 {
                    let change_pct = ((last.close - first.open) / first.open) * 100.0;
                    let min_low = data.iter().map(|d| d.low).fold(f64::INFINITY, f64::min);
                    let max_high = data.iter().map(|d| d.high).fold(f64::NEG_INFINITY, f64::max);
                    println!("📊 Change: {:+.2}% | Range: {:.0}-{:.0}", change_pct, min_low, max_high);
                }
            }
        }
        Err(e) => println!("❌ Failed to retrieve historical data: {:?}", e),
    }

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // 3. Test batch historical data
    println!("\n📊 Batch Historical Data (3 symbols - latest day)");
    println!("{}", "-".repeat(40));
    
    let test_symbols = vec!["VCI".to_string(), "TCB".to_string(), "FPT".to_string()];
    match client.get_batch_history(&test_symbols, "2025-08-14", Some("2025-08-15"), "1D").await {
        Ok(batch_data) => {
            println!("✅ Batch request successful for {} symbols!", test_symbols.len());
            println!("📈 Latest closing prices:");
            println!("{}", "-".repeat(40));
            
            for symbol in &test_symbols {
                if let Some(Some(data)) = batch_data.get(symbol) {
                    if let Some(latest) = data.last() {
                        println!("  {}: {:.0} VND", symbol, latest.close);
                    }
                } else {
                    println!("  {}: ❌ No data", symbol);
                }
            }
        }
        Err(e) => println!("❌ Batch request failed: {:?}", e),
    }

    println!("\n{}", "=".repeat(60));
    println!("✅ VCI CLIENT EXAMPLE COMPLETED");
    println!("{}", "=".repeat(60));

    Ok(())
}
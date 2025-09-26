# AIpriceaction - Vietnamese Stock Market Analysis Library

A comprehensive Rust library for Vietnamese stock market analysis featuring vectorized money flow calculations, MA score analysis, and AI prompt generation.

## Features

- 🚀 **High-Performance Analysis**: Vectorized money flow and MA score calculations
- 📊 **Vietnamese Market Focus**: Specialized for Vietnamese stock market data
- 🤖 **AI Integration**: Generate sophisticated analysis prompts with bilingual support
- 🏗️ **Builder Pattern**: Flexible configuration with sensible defaults
- 📈 **Multiple Analysis Types**: Money flow, MA scores, sector rotation, and more
- 🌐 **Bilingual Support**: English and Vietnamese templates and analysis

## Quick Start

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
aipriceaction = "0.1.0"
```

Basic usage:

```rust
use aipriceaction::prelude::*;
use aipriceaction::api::StockAnalyzer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let analyzer = StockAnalyzer::new();

    let result = analyzer.analyze_money_flow(
        vec!["VCB".to_string(), "BID".to_string()],
        DateRangeConfig::default_3m()
    ).await?;

    println!("Analysis complete: {} tickers processed", result.results.len());
    Ok(())
}
```

### As a CLI Tool

```bash
# Install the CLI
cargo install aipriceaction

# Generate AI analysis prompt
aipriceaction ask --tickers VCB,BID --template-id should-hold-sell-buy-more --language en

# Run data processing pipeline
aipriceaction run

# Check cache status
aipriceaction cache
```

## Library API

### High-Level API

The `StockAnalyzer` provides a simple interface for common tasks:

```rust
use aipriceaction::api::StockAnalyzer;

let analyzer = StockAnalyzer::new();

// Money flow analysis
let money_flow_result = analyzer.analyze_money_flow(tickers, date_range).await?;

// MA score analysis
let ma_score_result = analyzer.analyze_ma_score(tickers, ma_config).await?;

// AI prompt generation
let prompt = analyzer.generate_ai_prompt(tickers, ai_config).await?;
```

### Builder Pattern

Use the builder pattern for flexible configuration:

```rust
use aipriceaction::api::AnalysisBuilder;

// Pre-configured sector analysis
let banking_analyzer = AnalysisBuilder::banking_stocks()
    .medium_term()
    .with_ma_period(MAPeriod::MA20)
    .build();

// Custom configuration
let custom_analyzer = AnalysisBuilder::new()
    .add_ticker("VCB".to_string())
    .add_ticker("FPT".to_string())
    .with_date_range(DateRangeConfig::default_1y())
    .with_language(Language::Vietnamese)
    .build();
```

### Module Organization

- **`analysis`**: Core analysis functionality (money flow, MA scores, AI prompts)
- **`data`**: Data models and types
- **`api`**: High-level convenience API
- **`services`**: Reusable services (cache, data fetching)
- **`prelude`**: Common imports for convenience

## Examples

The `examples/` directory contains comprehensive examples:

```bash
# Basic analysis example
cargo run --example basic_analysis

# Builder pattern demonstration
cargo run --example builder_pattern

# AI prompt generation
cargo run --example ai_prompts
```

## Vietnamese Stock Market Support

### Sector Classifications

The library includes pre-configured ticker groups for major Vietnamese sectors:

- **Banking**: VCB, BID, CTG, TCB, MBB
- **Securities**: SSI, VCI, VCS, SHS, MBS
- **Real Estate**: VHM, VIC, VRE, NVL, KDH

### Analysis Features

- **Money Flow Analysis**: VNINDEX volume weighting and smart money detection
- **MA Score Calculation**: Moving average momentum analysis (MA10, MA20, MA50)
- **Sector Rotation**: Cross-sector momentum and rotation analysis
- **AI Prompt Generation**: 24+ specialized templates in English and Vietnamese

## AI Templates

### Available Templates

- `should-hold-sell-buy-more`: Investment decision analysis
- `money-flow-analysis`: Smart money vs retail analysis
- `market-leader-comparison`: Multi-ticker comparison
- `sector-rotation-analysis`: Cross-sector momentum
- `reversal-setup-scanner`: Technical reversal patterns

### Template Usage

```rust
use aipriceaction::api::analyzer::AIPromptConfig;

let config = AIPromptConfig {
    template_id: "should-hold-sell-buy-more".to_string(),
    language: Language::Vietnamese,
    chart_context_days: 14,
    ma_period: MAPeriod::MA20,
    ..Default::default()
};

let prompt = analyzer.generate_ai_prompt(tickers, config).await?;
```

## Performance

The library is optimized for performance with:

- **Vectorized Calculations**: SIMD-optimized mathematical operations
- **Intelligent Caching**: Multi-layer cache system with automatic invalidation
- **Parallel Processing**: Concurrent analysis of multiple tickers
- **Memory Efficiency**: Streaming data processing for large datasets

## CLI Commands

### `ask` - Generate AI Analysis Prompts

```bash
aipriceaction ask \
  --tickers VCB,BID,CTG \
  --template-id money-flow-analysis \
  --language vn \
  --chart-days 14 \
  --money-flow-days 10 \
  --ma-score-days 10 \
  --ma-period 20
```

### `run` - Data Processing Pipeline

```bash
# Run indefinitely (production mode)
aipriceaction run

# Run for specific number of ticks
aipriceaction run --ticks 100
```

### `cache` - Cache Management

```bash
aipriceaction cache
```

## Development

### Building

```bash
cargo build --release
```

### Running Tests

```bash
cargo test
```

### Running Examples

```bash
cargo run --example basic_analysis
```

## License

This project is dual-licensed under either:

- MIT License
- Apache License, Version 2.0

at your option.
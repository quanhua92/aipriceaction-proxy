//! # AIpriceaction - Vietnamese Stock Market Analysis Library
//!
//! A comprehensive Rust library for Vietnamese stock market analysis featuring:
//! - Vectorized money flow calculations
//! - MA (Moving Average) score analysis
//! - AI prompt generation for stock analysis
//! - High-performance data processing
//!
//! ## Quick Start
//!
//! ```rust
//! use aipriceaction::prelude::*;
//! use aipriceaction::api::StockAnalyzer;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let analyzer = StockAnalyzer::new();
//!     let result = analyzer.analyze_money_flow(
//!         vec!["VCB".to_string(), "BID".to_string()],
//!         DateRangeConfig::default_3m()
//!     ).await?;
//!     println!("Analysis complete: {} tickers processed", result.results.len());
//!     Ok(())
//! }
//! ```

// Core modules - these contain the main functionality
pub mod models;
pub mod utils;
pub mod services;

// Analysis modules - high-level analysis functionality
pub mod analysis {
    //! Core analysis functionality for Vietnamese stock market data

    /// Money flow analysis with vectorized calculations
    pub mod money_flow {
        pub use crate::utils::vectorized_money_flow::*;
        pub use crate::utils::money_flow_utils::*;
    }

    /// Moving Average score analysis
    pub mod ma_score {
        pub use crate::utils::vectorized_ma_score::*;
        pub use crate::models::ma_score::*;
    }

    /// AI prompt generation for stock analysis
    pub mod ask_ai {
        pub use crate::ask_ai::*;
    }

    /// Matrix utilities for vectorized operations
    pub mod matrix {
        pub use crate::utils::matrix_utils::*;
    }
}

// Data models and types
pub mod data {
    //! Data models and types used throughout the library

    pub use crate::models::stock_data::*;
    pub use crate::models::ticker::*;
    pub use crate::models::cache::*;

    /// Money flow related data structures
    pub mod money_flow {
        pub use crate::utils::money_flow_utils::{MoneyFlowTickerData, MoneyFlowResult, VolumeData, PerformanceMetrics};
    }

    /// MA score related data structures
    pub mod ma_score {
        pub use crate::models::ma_score::*;
    }
}

// Public API for easy library usage
pub mod api;

// Internal modules (not exposed in public API)
mod ask_ai;
pub mod state_machine;
mod states;

// Prelude for convenient imports
pub mod prelude {
    //! Prelude module for convenient imports
    //!
    //! Import this module to get the most commonly used types and functions:
    //! ```rust
    //! use aipriceaction::prelude::*;
    //! ```

    pub use crate::data::{StockDataPoint, DateRangeConfig, TickerGroups};
    pub use crate::data::money_flow::MoneyFlowTickerData;
    pub use crate::data::ma_score::{MAScoreTickerData, MAScoreResult};
    pub use crate::analysis::money_flow::{calculate_multiple_dates_vectorized, MoneyFlowResult};
    pub use crate::analysis::ma_score::{calculate_multiple_dates_vectorized_ma_score, MAScorePerformanceMetrics};
    pub use crate::services::CacheManager;
}

// Re-export some commonly used utilities
pub use utils::{init_logger, Logger, Timer};
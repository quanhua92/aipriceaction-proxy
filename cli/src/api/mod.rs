//! High-level API for easy library usage
//!
//! This module provides simplified interfaces for common stock analysis tasks.

pub mod builder;
pub mod analyzer;

pub use builder::AnalysisBuilder;
pub use analyzer::StockAnalyzer;
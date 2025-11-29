//! Red Team Benchmarking Infrastructure
//!
//! This module provides comprehensive metrics measurement, dashboard reporting, and
//! benchmark execution for evaluating the Intent Segregation Architecture against
//! state-of-the-art LLM security attacks.

pub mod metrics;
// TODO: pub mod dashboard;
// TODO: pub mod runners;
// TODO: pub mod datasets;

pub use metrics::{
    MetricsSnapshot, AggregatedMetrics,
    calculate_asr, calculate_frr, calculate_vault_detection_rate,
    calculate_parser_agreement_rate, calculate_voting_conflict_detection,
    calculate_policy_enforcement_accuracy, calculate_clean_utility,
    calculate_utility_under_attack, calculate_adaptive_asr, check_k_robustness,
    calculate_query_budget, calculate_avg_latency, calculate_p95_latency,
    calculate_p99_latency, calculate_throughput, calculate_token_overhead,
    LatencyTimer, VaultMetrics, ParserMetrics, VotingMetrics, ComparatorMetrics,
};

//! The Lexicanum Diagnostica - Health Monitoring System
//!
//! Monitors sentry health by feeding diagnostic prompts and detecting deviations from baseline.
//! Like a "sobriety test" - if a sentry's behavior changes dramatically, something is wrong.

use crate::diagnostics::{
    generate_diagnostic_suite, DiagnosticResult, ExpectedBehavior, SentryHealth,
    SentryHealthAssessment,
};
use crate::types::{BatchDiagnosticTest, SacrificialCogitator};
use std::time::{SystemTime, UNIX_EPOCH};

/// The Lexicanum Diagnostica - Health monitoring system for sentries
pub struct LexicanumDiagnostica {
    /// Baseline health scores to detect deviations
    baseline_scores: std::collections::HashMap<String, f32>,

    /// Deviation threshold before quarantine (0.5 = 50% deviation triggers alert)
    deviation_threshold: f32,

    /// Health score threshold for "Compromised" status (0.3 = below 30% = compromised)
    compromised_threshold: f32,

    /// Health score threshold for "Degraded" status (0.7 = below 70% = degraded)
    degraded_threshold: f32,
}

impl Default for LexicanumDiagnostica {
    fn default() -> Self {
        Self::new()
    }
}

impl LexicanumDiagnostica {
    /// Create a new health monitor
    pub fn new() -> Self {
        Self {
            baseline_scores: std::collections::HashMap::new(),
            deviation_threshold: 0.35, // 35% deviation triggers investigation
            compromised_threshold: 0.3, // Below 30% health = compromised
            degraded_threshold: 0.7,   // Below 70% health = degraded
        }
    }

    /// Set custom thresholds
    pub fn with_thresholds(mut self, degraded: f32, compromised: f32, deviation: f32) -> Self {
        self.degraded_threshold = degraded;
        self.compromised_threshold = compromised;
        self.deviation_threshold = deviation;
        self
    }

    /// Run full health assessment on a sentry
    pub async fn assess_sentry_health(
        &self,
        sentry: &dyn SacrificialCogitator,
    ) -> Result<SentryHealthAssessment, String> {
        let diagnostics = generate_diagnostic_suite();
        let mut results = Vec::new();
        let mut passed = 0;
        let mut findings = Vec::new();

        // Convert diagnostics to batch format for single API call
        let batch_tests: Vec<BatchDiagnosticTest> = diagnostics
            .iter()
            .map(|d| BatchDiagnosticTest {
                diagnostic_id: d.id.clone(),
                prompt: d.prompt.clone(),
            })
            .collect();

        // Run all diagnostics in single API call (90% cost reduction!)
        match sentry.test_batch_diagnostics(batch_tests).await {
            Ok(batch_response) => {
                for batch_result in batch_response.results {
                    // Find the matching diagnostic definition
                    if let Some(diagnostic) = diagnostics
                        .iter()
                        .find(|d| d.id == batch_result.diagnostic_id)
                    {
                        let reported_risk = batch_result.risk_score;

                        // Check if result matches expected behavior
                        let (matched, deviation) = match &diagnostic.expected_behavior {
                            ExpectedBehavior::ShouldFlagSuspicious { min_risk_score } => {
                                if reported_risk >= *min_risk_score {
                                    (true, 0.0)
                                } else {
                                    let deviation = (min_risk_score - reported_risk).abs();
                                    (false, deviation)
                                }
                            }
                            ExpectedBehavior::ShouldMarkClean { max_risk_score } => {
                                if reported_risk <= *max_risk_score {
                                    (true, 0.0)
                                } else {
                                    let deviation = (reported_risk - max_risk_score).abs();
                                    (false, deviation)
                                }
                            }
                        };

                        if matched {
                            passed += 1;
                        } else {
                            findings.push(format!(
                                "Test {} ({}): deviation of {:.2}",
                                diagnostic.id, diagnostic.description, deviation
                            ));
                        }

                        results.push(DiagnosticResult {
                            cogitator_name: sentry.cogitator_name(),
                            diagnostic_id: batch_result.diagnostic_id.clone(),
                            reported_risk_score: reported_risk,
                            passed: matched,
                            reason: diagnostic.description.clone(),
                            deviation_score: deviation,
                        });
                    }
                }
            }
            Err(e) => {
                // Fallback: if batching fails, report all tests as failed
                findings.push(format!(
                    "Batch diagnostic test failed: {} - falling back to individual tests",
                    e
                ));

                for diagnostic in &diagnostics {
                    results.push(DiagnosticResult {
                        cogitator_name: sentry.cogitator_name(),
                        diagnostic_id: diagnostic.id.clone(),
                        reported_risk_score: 0.5,
                        passed: false,
                        reason: format!("Batch failed: {}", e),
                        deviation_score: 1.0,
                    });
                }
            }
        }

        // Calculate overall health score
        let health_score = passed as f32 / diagnostics.len() as f32;

        // Determine health status
        let status = if health_score < self.compromised_threshold {
            SentryHealth::Compromised
        } else if health_score < self.degraded_threshold {
            SentryHealth::Degraded
        } else {
            SentryHealth::Healthy
        };

        // Add warning if compromised
        if status == SentryHealth::Compromised {
            findings.insert(
                0,
                format!(
                    "CRITICAL: Sentry health score {:.2}% - LIKELY COMPROMISED",
                    health_score * 100.0
                ),
            );
        } else if status == SentryHealth::Degraded {
            findings.insert(
                0,
                format!(
                    "WARNING: Sentry health score {:.2}% - DEGRADED performance detected",
                    health_score * 100.0
                ),
            );
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(SentryHealthAssessment {
            cogitator_name: sentry.cogitator_name(),
            health_score,
            tests_passed: passed,
            total_tests: diagnostics.len(),
            status,
            results,
            last_check_timestamp: now,
            findings,
        })
    }

    /// Check if a sentry has deviated significantly from baseline
    pub fn check_deviation(&self, sentry_name: &str, current_score: f32) -> (bool, f32) {
        match self.baseline_scores.get(sentry_name) {
            Some(&baseline) => {
                let deviation = (current_score - baseline).abs();
                let exceeded = deviation > self.deviation_threshold;
                (exceeded, deviation)
            }
            None => {
                // No baseline yet
                (false, 0.0)
            }
        }
    }

    /// Record baseline health score for comparison
    pub fn record_baseline(&mut self, sentry_name: String, health_score: f32) {
        self.baseline_scores.insert(sentry_name, health_score);
    }

    /// Get stored baseline for a sentry
    pub fn get_baseline(&self, sentry_name: &str) -> Option<f32> {
        self.baseline_scores.get(sentry_name).copied()
    }
}

/// Circuit breaker for a sentry - automatically quarantines unhealthy sentries
#[derive(Debug, Clone)]
pub struct SentryCircuitBreaker {
    /// Name of the sentry
    pub sentry_name: String,

    /// Current health status
    pub health: SentryHealth,

    /// Last health assessment
    pub last_assessment: Option<SentryHealthAssessment>,

    /// Number of consecutive failed health checks
    pub consecutive_failures: usize,

    /// Failure threshold before circuit opens (quarantine)
    pub failure_threshold: usize,

    /// Is this sentry quarantined?
    pub is_quarantined: bool,
}

impl SentryCircuitBreaker {
    /// Create a new circuit breaker for a sentry
    pub fn new(sentry_name: String) -> Self {
        Self {
            sentry_name,
            health: SentryHealth::Healthy,
            last_assessment: None,
            consecutive_failures: 0,
            failure_threshold: 2, // Quarantine after 2 failed health checks
            is_quarantined: false,
        }
    }

    /// Update circuit breaker with new health assessment
    pub fn update(&mut self, assessment: SentryHealthAssessment) {
        self.health = assessment.status;

        match assessment.status {
            SentryHealth::Compromised | SentryHealth::Dead => {
                self.consecutive_failures += 1;
                if self.consecutive_failures >= self.failure_threshold {
                    self.is_quarantined = true;
                    tracing::error!(
                        "CIRCUIT BREAKER: Sentry '{}' QUARANTINED after {} failures",
                        self.sentry_name,
                        self.consecutive_failures
                    );
                }
            }
            SentryHealth::Degraded => {
                self.consecutive_failures = self.consecutive_failures.saturating_add(1);
            }
            SentryHealth::Healthy => {
                // Reset on healthy
                self.consecutive_failures = 0;
                self.is_quarantined = false;
            }
        }

        self.last_assessment = Some(assessment);
    }

    /// Can this sentry be used for testing?
    pub fn is_usable(&self) -> bool {
        !self.is_quarantined && self.health != SentryHealth::Dead
    }

    /// Reset the circuit breaker (for testing)
    pub fn reset(&mut self) {
        self.consecutive_failures = 0;
        self.is_quarantined = false;
        self.health = SentryHealth::Healthy;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_monitor_creation() {
        let monitor = LexicanumDiagnostica::new();
        assert_eq!(monitor.degraded_threshold, 0.7);
        assert_eq!(monitor.compromised_threshold, 0.3);
    }

    #[test]
    fn test_circuit_breaker_creation() {
        let breaker = SentryCircuitBreaker::new("test_sentry".to_string());
        assert!(!breaker.is_quarantined);
        assert_eq!(breaker.consecutive_failures, 0);
        assert!(breaker.is_usable());
    }

    #[test]
    fn test_circuit_breaker_quarantine() {
        let mut breaker = SentryCircuitBreaker::new("test_sentry".to_string());

        // Create a compromised assessment
        let assessment = SentryHealthAssessment {
            cogitator_name: "test_sentry".to_string(),
            health_score: 0.2,
            tests_passed: 2,
            total_tests: 10,
            status: SentryHealth::Compromised,
            results: vec![],
            last_check_timestamp: 0,
            findings: vec![],
        };

        breaker.update(assessment.clone());
        assert_eq!(breaker.consecutive_failures, 1);
        assert!(!breaker.is_quarantined); // Not yet, threshold is 2

        breaker.update(assessment);
        assert_eq!(breaker.consecutive_failures, 2);
        assert!(breaker.is_quarantined); // Now quarantined
        assert!(!breaker.is_usable());
    }

    #[test]
    fn test_circuit_breaker_reset() {
        let mut breaker = SentryCircuitBreaker::new("test_sentry".to_string());
        breaker.consecutive_failures = 5;
        breaker.is_quarantined = true;

        breaker.reset();
        assert_eq!(breaker.consecutive_failures, 0);
        assert!(!breaker.is_quarantined);
        assert!(breaker.is_usable());
    }
}

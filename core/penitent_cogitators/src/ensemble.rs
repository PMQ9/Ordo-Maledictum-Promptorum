use crate::chatgpt::ChatGPTCogitator;
use crate::claude::ClaudeCogitator;
use crate::config::CogatorsConfig;
use crate::deepseek::DeepSeekCogitator;
use crate::types::{CogitatorCorruptionTest, CorruptionConsensus, SacrificialCogitator};
use std::sync::Arc;

/// Ensemble coordinator that runs all sacrificial LLMs in parallel
/// Produces consensus on input corruption/maliciousness
pub struct PenitentEnsemble {
    config: CogatorsConfig,
    chatgpt: Option<Arc<ChatGPTCogitator>>,
    deepseek: Option<Arc<DeepSeekCogitator>>,
    claude: Option<Arc<ClaudeCogitator>>,
}

impl PenitentEnsemble {
    /// Create a new ensemble from configuration
    pub fn from_config(config: CogatorsConfig) -> Self {
        let chatgpt = if config.enable_chatgpt && !config.chatgpt.api_key.is_empty() {
            Some(Arc::new(ChatGPTCogitator::new(config.chatgpt.clone())))
        } else {
            None
        };

        let deepseek = if config.enable_deepseek && !config.deepseek.api_key.is_empty() {
            Some(Arc::new(DeepSeekCogitator::new(config.deepseek.clone())))
        } else {
            None
        };

        let claude = if config.enable_claude && !config.claude.api_key.is_empty() {
            Some(Arc::new(ClaudeCogitator::new(config.claude.clone())))
        } else {
            None
        };

        Self {
            config,
            chatgpt,
            deepseek,
            claude,
        }
    }

    /// Create ensemble from environment variables
    pub fn from_env() -> Self {
        Self::from_config(CogatorsConfig::from_env())
    }

    /// Test input for corruption by running all configured cogitators in parallel
    pub async fn test_input_for_corruption(
        &self,
        user_input: &str,
    ) -> Result<CorruptionConsensus, Box<dyn std::error::Error>> {
        if user_input.trim().is_empty() {
            return Err("Empty input".into());
        }

        let mut handles = vec![];

        // Launch ChatGPT cogitator
        if let Some(chatgpt) = &self.chatgpt {
            let cogitator = Arc::clone(chatgpt);
            let input = user_input.to_string();
            let handle = tokio::spawn(async move { cogitator.test_for_corruption(&input).await });
            handles.push(("ChatGPT", handle));
        }

        // Launch DeepSeek cogitator
        if let Some(deepseek) = &self.deepseek {
            let cogitator = Arc::clone(deepseek);
            let input = user_input.to_string();
            let handle = tokio::spawn(async move { cogitator.test_for_corruption(&input).await });
            handles.push(("DeepSeek", handle));
        }

        // Launch Claude cogitator
        if let Some(claude) = &self.claude {
            let cogitator = Arc::clone(claude);
            let input = user_input.to_string();
            let handle = tokio::spawn(async move { cogitator.test_for_corruption(&input).await });
            handles.push(("Claude", handle));
        }

        if handles.is_empty() {
            return Err("No cogitators configured".into());
        }

        let total_cogitators = handles.len();

        // Collect results
        let mut results: Vec<CogitatorCorruptionTest> = vec![];
        let mut failed_cogitators = vec![];

        for (name, handle) in handles {
            match handle.await {
                Ok(Ok(result)) => {
                    results.push(result);
                }
                Ok(Err(e)) => {
                    tracing::warn!("Cogitator {} failed: {}", name, e);
                    failed_cogitators.push(format!("{}: {}", name, e));
                }
                Err(e) => {
                    tracing::warn!("Cogitator {} task failed: {}", name, e);
                    failed_cogitators.push(format!("{}: task error", name));
                }
            }
        }

        if results.is_empty() {
            return Err(format!("All cogitators failed: {}", failed_cogitators.join("; ")).into());
        }

        // Calculate consensus
        let is_corrupted = if self.config.require_consensus {
            // All must agree input is clean
            results.iter().all(|r| !r.is_suspicious)
        } else {
            // Any flagging as suspicious means it's suspicious
            results.iter().any(|r| r.is_suspicious)
        };

        let suspicious_count = results.iter().filter(|r| r.is_suspicious).count();
        let consensus_risk_score = if results.is_empty() {
            0.0
        } else {
            results.iter().map(|r| r.risk_score).sum::<f32>() / results.len() as f32
        };

        // Combine analysis
        let combined_analysis = results
            .iter()
            .map(|r| {
                format!(
                    "{}: {} (risk: {:.2})",
                    r.cogitator_name, r.analysis, r.risk_score
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        tracing::info!(
            "Ensemble consensus: corrupted={}, risk={:.2}, suspicious={}/{}",
            is_corrupted,
            consensus_risk_score,
            suspicious_count,
            total_cogitators
        );

        Ok(CorruptionConsensus {
            is_corrupted,
            consensus_risk_score,
            suspicious_count,
            total_cogitators,
            individual_results: results,
            combined_analysis,
        })
    }

    /// Check if any cogitators are configured
    pub fn is_configured(&self) -> bool {
        self.chatgpt.is_some() || self.deepseek.is_some() || self.claude.is_some()
    }

    /// Get count of configured cogitators
    pub fn configured_count(&self) -> usize {
        let mut count = 0;
        if self.chatgpt.is_some() {
            count += 1;
        }
        if self.deepseek.is_some() {
            count += 1;
        }
        if self.claude.is_some() {
            count += 1;
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_config_not_configured() {
        let config = CogatorsConfig {
            enable_chatgpt: false,
            enable_deepseek: false,
            enable_claude: false,
            ..Default::default()
        };

        let ensemble = PenitentEnsemble::from_config(config);
        assert!(!ensemble.is_configured());
        assert_eq!(ensemble.configured_count(), 0);
    }

    #[test]
    fn test_partial_config() {
        let mut config = CogatorsConfig::default();
        config.enable_chatgpt = true;
        config.enable_deepseek = false;
        config.enable_claude = false;
        config.chatgpt.api_key = "test_key".to_string();

        let ensemble = PenitentEnsemble::from_config(config);
        assert!(ensemble.is_configured());
        assert_eq!(ensemble.configured_count(), 1);
    }

    #[test]
    fn test_all_configured() {
        let mut config = CogatorsConfig::default();
        config.chatgpt.api_key = "test_key_1".to_string();
        config.deepseek.api_key = "test_key_2".to_string();
        config.claude.api_key = "test_key_3".to_string();

        let ensemble = PenitentEnsemble::from_config(config);
        assert!(ensemble.is_configured());
        assert_eq!(ensemble.configured_count(), 3);
    }
}

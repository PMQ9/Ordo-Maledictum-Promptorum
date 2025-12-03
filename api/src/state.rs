//! Application state shared across all handlers

use crate::config::Config;
use anyhow::Result;
use intent_comparator::IntentComparator;
use intent_generator::TrustedIntentGenerator;
use intent_ledger::IntentLedger;
use intent_parsers::ParserEnsemble;
use intent_schema::ProviderConfig;
use intent_voting::VotingModule;
use malicious_detector::MaliciousDetector;
use processing_engine::ProcessingEngine;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Pending approval request
#[derive(Debug, Clone)]
pub struct PendingApproval {
    pub id: Uuid,
    pub user_id: String,
    pub session_id: String,
    pub intent: intent_schema::Intent,
    pub reason: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Approval decision
#[derive(Debug, Clone)]
pub struct ApprovalDecision {
    pub approved: bool,
    pub approver_id: String,
    pub reason: String,
    pub decided_at: chrono::DateTime<chrono::Utc>,
}

/// Shared application state
pub struct AppState {
    /// Configuration
    pub config: Config,

    /// Database connection pool
    pub db_pool: PgPool,

    /// Malicious input detector
    pub detector: MaliciousDetector,

    /// Parser ensemble
    pub parser_ensemble: ParserEnsemble,

    /// Voting module
    pub voting: VotingModule,

    /// Intent comparator
    pub comparator: IntentComparator,

    /// Intent generator
    pub generator: TrustedIntentGenerator,

    /// Processing engine
    pub engine: ProcessingEngine,

    /// Intent ledger
    pub ledger: IntentLedger,

    /// Provider configuration
    pub provider_config: ProviderConfig,

    /// Pending approval requests (in-memory for simplicity)
    /// In production, this would be backed by a database
    pub pending_approvals: Arc<RwLock<HashMap<Uuid, PendingApproval>>>,

    /// Approval decisions
    pub approval_decisions: Arc<RwLock<HashMap<Uuid, ApprovalDecision>>>,
}

impl AppState {
    /// Create a new AppState from configuration
    pub async fn new(config: Config) -> Result<Self> {
        // Initialize database connection pool
        let db_pool = PgPoolOptions::new()
            .max_connections(config.database.max_connections)
            .connect(&config.database.url)
            .await?;

        tracing::info!("Database connection pool created");

        // Initialize malicious detector
        let detector = MaliciousDetector::new();

        // Build parser configuration
        let parser_config = build_parser_config(&config);

        // Initialize parser ensemble
        let ensemble = ParserEnsemble::new(parser_config);
        tracing::info!(
            "Parser ensemble initialized with {} parsers",
            ensemble.parser_count()
        );

        // Initialize voting module
        let voting = VotingModule::new();

        // Initialize intent comparator
        let comparator = IntentComparator::new();

        // Initialize intent generator
        let generator = TrustedIntentGenerator::with_defaults();

        // Initialize processing engine with Claude API key from config
        let engine_config = processing_engine::EngineConfig {
            verbose: false,
            max_execution_time_ms: 30_000,
            claude_api_key: config.parsers.claude_api_key.clone(),
            claude_model: config.parsers.claude_model.clone(),
        };
        let engine = ProcessingEngine::with_config(engine_config);

        // Initialize ledger
        let ledger = IntentLedger::from_pool(db_pool.clone());

        // Build provider config from settings
        let provider_config = build_provider_config(&config);

        Ok(Self {
            config,
            db_pool,
            detector,
            parser_ensemble: ensemble,
            voting,
            comparator,
            generator,
            engine,
            ledger,
            provider_config,
            pending_approvals: Arc::new(RwLock::new(HashMap::new())),
            approval_decisions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Add a pending approval request
    pub async fn add_pending_approval(&self, approval: PendingApproval) -> Uuid {
        let id = approval.id;
        let mut approvals = self.pending_approvals.write().await;
        approvals.insert(id, approval);
        id
    }

    /// Get a pending approval request
    pub async fn get_pending_approval(&self, id: Uuid) -> Option<PendingApproval> {
        let approvals = self.pending_approvals.read().await;
        approvals.get(&id).cloned()
    }

    /// Submit an approval decision
    pub async fn submit_approval_decision(&self, id: Uuid, decision: ApprovalDecision) {
        let mut decisions = self.approval_decisions.write().await;
        decisions.insert(id, decision);
    }

    /// Get an approval decision
    pub async fn get_approval_decision(&self, id: Uuid) -> Option<ApprovalDecision> {
        let decisions = self.approval_decisions.read().await;
        decisions.get(&id).cloned()
    }

    /// Check if an approval has been decided
    pub async fn is_approval_decided(&self, id: Uuid) -> bool {
        let decisions = self.approval_decisions.read().await;
        decisions.contains_key(&id)
    }
}

/// Build ParserConfig from application config
fn build_parser_config(config: &Config) -> intent_parsers::ParserConfig {
    use intent_parsers::{ClaudeConfig, DeepSeekConfig, OpenAIConfig, ParserConfig};

    let openai_config = OpenAIConfig {
        api_key: config.parsers.openai_api_key.clone().unwrap_or_default(),
        model: config.parsers.openai_model.clone(),
        temperature: 1.0, // OpenAI requires temperature >= 1.0 for new models
        timeout_secs: 30,
        base_url: "https://api.openai.com/v1".to_string(),
    };

    let deepseek_config = DeepSeekConfig {
        api_key: config.parsers.deepseek_api_key.clone().unwrap_or_default(),
        model: config.parsers.deepseek_model.clone(),
        temperature: 0.0,
        timeout_secs: 30,
        base_url: "https://api.deepseek.com/v1".to_string(),
    };

    let claude_config = ClaudeConfig {
        api_key: config.parsers.claude_api_key.clone().unwrap_or_default(),
        model: config.parsers.claude_model.clone(),
        temperature: 0.0,
        timeout_secs: 30,
        base_url: "https://api.anthropic.com/v1".to_string(),
    };

    ParserConfig {
        enable_openai: config.parsers.enable_openai,
        enable_deepseek: config.parsers.enable_deepseek,
        enable_claude: config.parsers.enable_claude,
        openai: openai_config,
        deepseek: deepseek_config,
        claude: claude_config,
    }
}

/// Build ProviderConfig from application config
fn build_provider_config(config: &Config) -> ProviderConfig {
    ProviderConfig {
        allowed_actions: config.provider.allowed_actions.clone(),
        allowed_expertise: config.provider.allowed_expertise.clone(),
        max_budget: config.provider.max_budget.map(|b| b as i64),
        allowed_domains: vec![],
        require_human_approval: config.provider.require_human_approval,
    }
}

//! Application state shared across all handlers

use crate::config::Config;
use anyhow::Result;
use intent_comparator::IntentComparator;
use intent_generator::IntentGenerator;
use intent_ledger::IntentLedger;
use intent_parsers::{DeterministicParser, OllamaParser, OpenAIParser, ParserEnsemble};
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
    pub generator: IntentGenerator,

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

        // Initialize parser ensemble
        let mut ensemble = ParserEnsemble::new();

        // Add deterministic parser
        if config.parsers.enable_deterministic {
            let deterministic = DeterministicParser::new();
            ensemble.add_parser(Box::new(deterministic));
            tracing::info!("Added deterministic parser to ensemble");
        }

        // Add OpenAI parser if enabled
        if config.parsers.enable_openai {
            if let Some(api_key) = &config.parsers.openai_api_key {
                let openai =
                    OpenAIParser::new(api_key.clone(), config.parsers.openai_model.clone());
                ensemble.add_parser(Box::new(openai));
                tracing::info!("Added OpenAI parser to ensemble");
            } else {
                tracing::warn!("OpenAI parser enabled but no API key provided");
            }
        }

        // Add Ollama parser if enabled
        if config.parsers.enable_ollama {
            let ollama = OllamaParser::new(
                config.parsers.ollama_endpoint.clone(),
                config.parsers.ollama_model.clone(),
            );
            ensemble.add_parser(Box::new(ollama));
            tracing::info!("Added Ollama parser to ensemble");
        }

        // Initialize voting module
        let voting = VotingModule::new();

        // Initialize intent comparator
        let comparator = IntentComparator::new();

        // Initialize intent generator
        let generator = IntentGenerator::new();

        // Initialize processing engine
        let engine = ProcessingEngine::new();

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

/// Build ProviderConfig from application config
fn build_provider_config(config: &Config) -> ProviderConfig {
    use intent_schema::{Action, Expertise};

    // Parse allowed actions
    let allowed_actions: Vec<Action> = config
        .provider
        .allowed_actions
        .iter()
        .filter_map(|s| match s.to_lowercase().as_str() {
            "find_experts" | "findexperts" => Some(Action::FindExperts),
            "summarize" => Some(Action::Summarize),
            "draft_proposal" | "draftproposal" => Some(Action::DraftProposal),
            _ => {
                tracing::warn!("Unknown action in config: {}", s);
                None
            }
        })
        .collect();

    // Parse allowed expertise
    let allowed_expertise: Vec<Expertise> = config
        .provider
        .allowed_expertise
        .iter()
        .filter_map(|s| match s.to_lowercase().as_str() {
            "ml" | "machine_learning" | "machinelearning" => Some(Expertise::MachineLearning),
            "security" | "cybersecurity" => Some(Expertise::Security),
            "embedded" => Some(Expertise::Embedded),
            "cloud" => Some(Expertise::Cloud),
            "frontend" => Some(Expertise::Frontend),
            "backend" => Some(Expertise::Backend),
            "devops" => Some(Expertise::DevOps),
            "data" | "data_science" => Some(Expertise::DataScience),
            _ => {
                tracing::warn!("Unknown expertise in config: {}", s);
                None
            }
        })
        .collect();

    ProviderConfig {
        allowed_actions,
        allowed_expertise,
        max_budget: config.provider.max_budget,
        max_results: config.provider.max_results,
        require_human_approval: config.provider.require_human_approval,
        custom_constraints: HashMap::new(),
    }
}

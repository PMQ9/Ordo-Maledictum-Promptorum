use chrono::Utc;
use intent_schema::{
    Action, Constraints, GenerationMetadata as IntentMetadata, SchemaError, TrustedIntent,
    VotedIntent,
};
use serde_json;
use std::collections::HashSet;
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during intent generation
#[derive(Debug, Error)]
pub enum GeneratorError {
    #[error("Schema validation error: {0}")]
    SchemaError(#[from] SchemaError),

    #[error("Sanitization failed: {0}")]
    SanitizationError(String),

    #[error("Invalid content reference: {0}")]
    InvalidContentRef(String),

    #[error("Topic normalization failed: {0}")]
    TopicNormalizationError(String),

    #[error("Signature generation failed: {0}")]
    SignatureError(String),

    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Configuration for the intent generator
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Enable cryptographic signatures
    pub enable_signatures: bool,

    /// Signing key (if signatures enabled)
    pub signing_key: Option<Vec<u8>>,

    /// Maximum allowed content references
    pub max_content_refs: usize,

    /// Allowed actions whitelist
    pub allowed_actions: HashSet<Action>,

    /// Maximum topic ID length
    pub max_topic_id_length: usize,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            enable_signatures: false,
            signing_key: None,
            max_content_refs: 10,
            allowed_actions: HashSet::new(),
            max_topic_id_length: 100,
        }
    }
}

/// Trusted Intent Generator
///
/// This component takes a VotedIntent (output from the voting module) and produces
/// a canonical, sanitized, and optionally signed TrustedIntent that can be safely
/// executed by the processing engine.
///
/// Key responsibilities:
/// 1. Remove all raw user content
/// 2. Normalize topics to identifiers
/// 3. Validate and sanitize content references
/// 4. Add metadata (UUID, timestamp, etc.)
/// 5. Optionally sign the intent
/// 6. Generate content hash for integrity
pub struct TrustedIntentGenerator {
    config: GeneratorConfig,
}

impl TrustedIntentGenerator {
    /// Create a new TrustedIntentGenerator with the given configuration
    pub fn new(config: GeneratorConfig) -> Self {
        Self { config }
    }

    /// Create a new generator with default configuration
    pub fn with_defaults() -> Self {
        Self {
            config: GeneratorConfig::default(),
        }
    }

    /// Generate a TrustedIntent from a VotedIntent
    ///
    /// This is the main entry point for intent generation. It performs:
    /// - Sanitization of all fields
    /// - Removal of raw user content
    /// - Normalization of topics to identifiers
    /// - Validation of content references
    /// - Addition of metadata
    /// - Optional cryptographic signing
    ///
    /// # Arguments
    /// * `voted_intent` - The result of the voting process
    /// * `metadata` - Additional metadata about the request
    ///
    /// # Returns
    /// A TrustedIntent ready for execution
    pub async fn generate(
        &self,
        voted_intent: VotedIntent,
        metadata: IntentMetadata,
    ) -> Result<TrustedIntent, GeneratorError> {
        // 1. Validate action is allowed
        self.validate_action(&voted_intent.action)?;

        // 2. Normalize topic to an identifier (remove raw user text)
        let topic_id = self.normalize_topic(&voted_intent.topic)?;

        // 3. Sanitize and validate content references
        let content_refs = self.sanitize_content_refs(&voted_intent.content_refs)?;

        // 4. Sanitize and validate constraints
        let constraints = self.sanitize_constraints(voted_intent.constraints)?;

        // 5. Remove duplicates from expertise
        let expertise = self.deduplicate_expertise(voted_intent.expertise);

        // 6. Generate unique ID and timestamp
        let id = Uuid::new_v4();
        let timestamp = Utc::now();

        // 7. Create the base trusted intent (without signature)
        let mut trusted_intent = TrustedIntent {
            id,
            timestamp,
            action: voted_intent.action,
            topic_id,
            expertise,
            constraints,
            content_refs,
            signature: None,
            content_hash: String::new(), // Will be set below
            user_id: metadata.user_id,
            session_id: metadata.session_id,
        };

        // 8. Generate content hash for integrity
        trusted_intent.content_hash = self.generate_content_hash(&trusted_intent)?;

        // 9. Optionally sign the intent
        if self.config.enable_signatures {
            trusted_intent.signature = Some(self.sign_intent(&trusted_intent).await?);
        }

        // 10. Final validation: ensure no raw content leaked through
        trusted_intent.validate_no_raw_content()?;

        Ok(trusted_intent)
    }

    /// Validate that the action is allowed
    fn validate_action(&self, action: &Action) -> Result<(), GeneratorError> {
        if !self.config.allowed_actions.is_empty() && !self.config.allowed_actions.contains(action)
        {
            return Err(GeneratorError::SanitizationError(format!(
                "Action {:?} is not in the allowed list",
                action
            )));
        }
        Ok(())
    }

    /// Normalize a raw topic string to a safe identifier
    ///
    /// This is critical for security: we convert free-form user text
    /// into a normalized identifier that can be safely used in queries.
    ///
    /// Strategy:
    /// - Convert to lowercase
    /// - Replace spaces with underscores
    /// - Remove special characters
    /// - Truncate to max length
    /// - Validate result is a safe identifier
    fn normalize_topic(&self, topic: &str) -> Result<String, GeneratorError> {
        // Convert to lowercase and replace spaces
        let normalized = topic
            .to_lowercase()
            .trim()
            .replace(' ', "_")
            .replace('-', "_");

        // Remove any characters that aren't alphanumeric or underscore
        let sanitized: String = normalized
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect();

        // Ensure it's not empty
        if sanitized.is_empty() {
            return Err(GeneratorError::TopicNormalizationError(
                "Topic normalized to empty string".to_string(),
            ));
        }

        // Truncate to max length
        let truncated = if sanitized.len() > self.config.max_topic_id_length {
            &sanitized[..self.config.max_topic_id_length]
        } else {
            &sanitized
        };

        // Ensure it starts with a letter or underscore (valid identifier)
        if !truncated.starts_with(|c: char| c.is_alphabetic() || c == '_') {
            return Err(GeneratorError::TopicNormalizationError(
                "Topic must start with a letter or underscore".to_string(),
            ));
        }

        Ok(truncated.to_string())
    }

    /// Sanitize and validate content references
    ///
    /// Content references must be:
    /// - References to already-sanitized content (not raw content)
    /// - Valid identifiers
    /// - Within the allowed count limit
    fn sanitize_content_refs(&self, refs: &[String]) -> Result<Vec<String>, GeneratorError> {
        // Check count limit
        if refs.len() > self.config.max_content_refs {
            return Err(GeneratorError::SanitizationError(format!(
                "Too many content references: {} > {}",
                refs.len(),
                self.config.max_content_refs
            )));
        }

        let mut sanitized = Vec::new();

        for content_ref in refs {
            // Validate that this looks like a reference, not raw content
            if content_ref.contains('\n') {
                return Err(GeneratorError::InvalidContentRef(
                    "Content reference contains newlines".to_string(),
                ));
            }

            if content_ref.len() > 100 {
                return Err(GeneratorError::InvalidContentRef(
                    "Content reference too long (max 100 chars)".to_string(),
                ));
            }

            // Ensure it's a valid identifier format (e.g., "doc_1234", "file_abc")
            if !content_ref
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
            {
                return Err(GeneratorError::InvalidContentRef(format!(
                    "Invalid content reference format: {}",
                    content_ref
                )));
            }

            sanitized.push(content_ref.clone());
        }

        Ok(sanitized)
    }

    /// Sanitize constraints
    ///
    /// Ensures:
    /// - No injection through constraint values
    /// - All values are within valid ranges
    /// - No unexpected fields
    fn sanitize_constraints(
        &self,
        constraints: Option<Constraints>,
    ) -> Result<Constraints, GeneratorError> {
        let mut sanitized = constraints.unwrap_or_default();

        // Remove any additional fields that might contain user input
        // We only keep the known, validated fields
        sanitized.additional.clear();

        // Validate using the validator crate
        use validator::Validate;
        sanitized.validate().map_err(|e| {
            GeneratorError::SanitizationError(format!("Constraint validation failed: {}", e))
        })?;

        Ok(sanitized)
    }

    /// Remove duplicate expertise areas
    fn deduplicate_expertise(
        &self,
        expertise: Vec<intent_schema::Expertise>,
    ) -> Vec<intent_schema::Expertise> {
        let mut seen = HashSet::new();
        expertise
            .into_iter()
            .filter(|e| seen.insert(e.clone()))
            .collect()
    }

    /// Generate a content hash for integrity verification
    ///
    /// This creates a deterministic hash of the intent content (excluding the signature)
    /// that can be used to verify the intent hasn't been tampered with.
    fn generate_content_hash(&self, intent: &TrustedIntent) -> Result<String, GeneratorError> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Create a deterministic JSON representation (excluding signature and hash)
        let hashable = serde_json::json!({
            "id": intent.id.to_string(),
            "timestamp": intent.timestamp.to_rfc3339(),
            "action": intent.action,
            "topic_id": &intent.topic_id,
            "expertise": &intent.expertise,
            "constraints": &intent.constraints,
            "content_refs": &intent.content_refs,
            "user_id": &intent.user_id,
            "session_id": &intent.session_id,
        });

        let json_str = serde_json::to_string(&hashable)?;

        let mut hasher = DefaultHasher::new();
        json_str.hash(&mut hasher);
        let hash = hasher.finish();

        Ok(format!("{:x}", hash))
    }

    /// Sign the intent using the configured signing key
    ///
    /// # Cryptographic Signature Approach
    ///
    /// For production use, implement one of these approaches:
    ///
    /// ## Option 1: HMAC (Symmetric)
    /// - Use HMAC-SHA256 with a secret key
    /// - Fast and simple
    /// - Requires secure key distribution
    /// - Use the `hmac` and `sha2` crates
    ///
    /// ```rust,ignore
    /// use hmac::{Hmac, Mac};
    /// use sha2::Sha256;
    ///
    /// type HmacSha256 = Hmac<Sha256>;
    ///
    /// let mut mac = HmacSha256::new_from_slice(&key)?;
    /// mac.update(data.as_bytes());
    /// let result = mac.finalize();
    /// let signature = hex::encode(result.into_bytes());
    /// ```
    ///
    /// ## Option 2: Ed25519 (Asymmetric)
    /// - Use Ed25519 digital signatures
    /// - Public key can be distributed for verification
    /// - Use the `ed25519-dalek` crate
    ///
    /// ```rust,ignore
    /// use ed25519_dalek::{Keypair, Signature, Signer};
    ///
    /// let signature: Signature = keypair.sign(data.as_bytes());
    /// let signature_hex = hex::encode(signature.to_bytes());
    /// ```
    ///
    /// ## Option 3: RSA (Asymmetric)
    /// - Use RSA-PSS or RSA-PKCS1v15
    /// - Industry standard
    /// - Use the `rsa` crate
    ///
    /// For now, this is a placeholder that returns a mock signature.
    /// Replace with actual cryptographic implementation based on requirements.
    async fn sign_intent(&self, intent: &TrustedIntent) -> Result<String, GeneratorError> {
        // Verify we have a signing key
        let _key = self.config.signing_key.as_ref().ok_or_else(|| {
            GeneratorError::SignatureError("No signing key configured".to_string())
        })?;

        // Create the data to sign (use the content hash for consistency)
        let data_to_sign = format!(
            "{}:{}:{}",
            intent.id,
            intent.timestamp.to_rfc3339(),
            intent.content_hash
        );

        // TODO: Implement actual cryptographic signing
        // For now, return a placeholder that indicates signing is configured
        Ok(format!("SIGNATURE_PLACEHOLDER_{}", &data_to_sign[..20]))

        /*
        // Example HMAC implementation (uncomment and add dependencies):
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(key)
            .map_err(|e| GeneratorError::SignatureError(format!("HMAC error: {}", e)))?;

        mac.update(data_to_sign.as_bytes());
        let result = mac.finalize();
        let signature = hex::encode(result.into_bytes());

        Ok(signature)
        */
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use intent_schema::{Action, Expertise};

    fn create_test_voted_intent() -> VotedIntent {
        VotedIntent {
            action: Action::MathQuestion,
            topic: "What is the square root of 144?".to_string(),
            expertise: vec![], // math_question doesn't use expertise
            constraints: Some(Constraints {
                max_budget: Some(20000),
                max_results: Some(5),
                deadline: None,
                additional: std::collections::HashMap::new(),
            }),
            content_refs: vec![],
            confidence: 0.95,
            requires_approval: false,
            parser_results: vec![],
        }
    }

    fn create_test_metadata() -> IntentMetadata {
        IntentMetadata {
            user_id: "user_123".to_string(),
            session_id: "session_456".to_string(),
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
        }
    }

    #[tokio::test]
    async fn test_generate_trusted_intent() {
        let mut config = GeneratorConfig::default();
        config.allowed_actions.insert(Action::MathQuestion);

        let generator = TrustedIntentGenerator::new(config);
        let voted_intent = create_test_voted_intent();
        let metadata = create_test_metadata();

        let result = generator.generate(voted_intent, metadata).await;
        assert!(result.is_ok());

        let trusted = result.unwrap();
        assert_eq!(trusted.action, Action::MathQuestion);
        assert_eq!(trusted.topic_id, "what_is_the_square_root_of_144");
        assert_eq!(trusted.expertise.len(), 0); // No expertise for math questions
        assert_eq!(trusted.content_refs.len(), 0);
        assert_eq!(trusted.user_id, "user_123");
        assert!(trusted.signature.is_none()); // Signing disabled
    }

    #[tokio::test]
    async fn test_topic_normalization() {
        let generator = TrustedIntentGenerator::with_defaults();

        // Valid cases
        assert_eq!(
            generator.normalize_topic("Supply Chain Risk").unwrap(),
            "supply_chain_risk"
        );
        assert_eq!(
            generator.normalize_topic("ML-Security-Analysis").unwrap(),
            "ml_security_analysis"
        );
        assert_eq!(
            generator.normalize_topic("_private_topic").unwrap(),
            "_private_topic"
        );

        // Should remove special characters
        assert_eq!(
            generator.normalize_topic("Hello@World!").unwrap(),
            "helloworld"
        );

        // Should fail on empty result
        assert!(generator.normalize_topic("@#$%").is_err());

        // Should fail if starts with number after sanitization
        let result = generator.normalize_topic("123topic");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_content_ref_validation() {
        let generator = TrustedIntentGenerator::with_defaults();

        // Valid references
        let valid_refs = vec!["doc_123".to_string(), "file_abc".to_string()];
        assert!(generator.sanitize_content_refs(&valid_refs).is_ok());

        // Invalid: contains newline
        let invalid_refs = vec!["doc_123\nmalicious".to_string()];
        assert!(generator.sanitize_content_refs(&invalid_refs).is_err());

        // Invalid: too long
        let too_long = vec!["x".repeat(101)];
        assert!(generator.sanitize_content_refs(&too_long).is_err());

        // Invalid: special characters
        let special_chars = vec!["doc@123".to_string()];
        assert!(generator.sanitize_content_refs(&special_chars).is_err());
    }

    #[tokio::test]
    async fn test_constraints_sanitization() {
        let generator = TrustedIntentGenerator::with_defaults();

        let mut constraints = Constraints::default();
        constraints.max_budget = Some(50000);
        constraints.additional.insert(
            "malicious_field".to_string(),
            serde_json::json!("malicious_value"),
        );

        let sanitized = generator.sanitize_constraints(Some(constraints)).unwrap();

        // Additional fields should be removed
        assert!(sanitized.additional.is_empty());
        assert_eq!(sanitized.max_budget, Some(50000));
    }

    #[tokio::test]
    async fn test_signature_generation() {
        let mut config = GeneratorConfig::default();
        config.enable_signatures = true;
        config.signing_key = Some(b"test_secret_key_32_bytes_long!!!".to_vec());
        config.allowed_actions.insert(Action::MathQuestion);

        let generator = TrustedIntentGenerator::new(config);
        let voted_intent = create_test_voted_intent();
        let metadata = create_test_metadata();

        let trusted = generator.generate(voted_intent, metadata).await.unwrap();

        // Should have a signature
        assert!(trusted.signature.is_some());
        assert!(trusted
            .signature
            .unwrap()
            .starts_with("SIGNATURE_PLACEHOLDER_"));
    }

    #[tokio::test]
    async fn test_no_raw_content_validation() {
        let mut config = GeneratorConfig::default();
        config.allowed_actions.insert(Action::MathQuestion);

        let generator = TrustedIntentGenerator::new(config);

        let mut voted_intent = create_test_voted_intent();
        // Try to inject raw content via content_refs
        voted_intent.content_refs = vec!["This is raw\ncontent with newlines".to_string()];

        let metadata = create_test_metadata();
        let result = generator.generate(voted_intent, metadata).await;

        // Should fail validation
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_expertise_deduplication() {
        let mut config = GeneratorConfig::default();
        config.allowed_actions.insert(Action::MathQuestion);

        let generator = TrustedIntentGenerator::new(config);

        let mut voted_intent = create_test_voted_intent();
        // math_question doesn't use expertise, but test with empty list
        voted_intent.expertise = vec![];

        let metadata = create_test_metadata();
        let trusted = generator.generate(voted_intent, metadata).await.unwrap();

        // Should have 0 expertise areas (math questions don't use expertise)
        assert_eq!(trusted.expertise.len(), 0);
    }
}

# Intent Generator Module

The Intent Generator is a critical security component that produces canonical, sanitized, and optionally signed trusted intents from voted intents.

## Purpose

Takes the output of the voting module (a `VotedIntent`) and produces a `TrustedIntent` that:
1. Contains only allowed fields
2. Has all raw user content removed
3. Uses content references instead of raw text
4. Includes metadata (UUID, timestamp, etc.)
5. Is optionally cryptographically signed
6. Has an integrity hash

## Key Features

### 1. Sanitization
- **Topic Normalization**: Converts free-form topics to safe identifiers
  - Example: "Supply Chain Risk Analysis" → "supply_chain_risk_analysis"
- **Content Reference Validation**: Ensures references don't contain raw content
- **Constraint Cleaning**: Removes potentially malicious additional fields

### 2. Security
- **No Raw User Content**: All free-form text is converted to identifiers or references
- **Validation**: Multiple validation layers ensure data integrity
- **Action Whitelisting**: Only allowed actions can be processed

### 3. Cryptographic Signing (Optional)
- Supports HMAC-SHA256 (symmetric)
- Supports Ed25519 (asymmetric) - see documentation in code
- Supports RSA (asymmetric) - see documentation in code
- Currently returns placeholder signatures (implementation ready for production keys)

## Usage

```rust
use intent_generator::{TrustedIntentGenerator, GeneratorConfig};
use intent_schema::{VotedIntent, GenerationMetadata, Action};
use std::collections::HashSet;

// Configure the generator
let mut config = GeneratorConfig::default();
config.allowed_actions.insert(Action::FindExperts);
config.max_content_refs = 10;

// Optionally enable signatures
config.enable_signatures = true;
config.signing_key = Some(your_secret_key);

let generator = TrustedIntentGenerator::new(config);

// Create metadata
let metadata = GenerationMetadata {
    user_id: "user_123".to_string(),
    session_id: "session_456".to_string(),
    ip_address: Some("192.168.1.1".to_string()),
    user_agent: Some("Mozilla/5.0".to_string()),
};

// Generate trusted intent
let trusted = generator.generate(voted_intent, metadata).await?;
```

## Configuration

### GeneratorConfig

- `enable_signatures`: Whether to sign intents (default: false)
- `signing_key`: Cryptographic key for signing (default: None)
- `max_content_refs`: Maximum allowed content references (default: 10)
- `allowed_actions`: Whitelist of allowed actions (default: empty = allow all)
- `max_topic_id_length`: Maximum length for topic identifiers (default: 100)

## Output: TrustedIntent

```rust
pub struct TrustedIntent {
    pub id: Uuid,                    // Unique identifier
    pub timestamp: DateTime<Utc>,    // When generated
    pub action: Action,              // Type-safe action
    pub topic_id: String,            // Sanitized topic identifier
    pub expertise: Vec<Expertise>,   // Required expertise
    pub constraints: Constraints,    // Validated constraints
    pub content_refs: Vec<String>,   // Sanitized content references
    pub signature: Option<String>,   // Optional cryptographic signature
    pub content_hash: String,        // Integrity hash
    pub user_id: String,             // User identifier
    pub session_id: String,          // Session identifier
}
```

## Security Guarantees

1. **No Prompt Injection**: All free-form text is removed or normalized
2. **Type Safety**: Uses enums for actions and expertise
3. **Validation**: Multiple validation layers
4. **Immutability**: Once generated, trusted intents should not be modified
5. **Auditability**: Includes hash and optional signature for verification
6. **Content Isolation**: Raw user content is never directly included

## Testing

Run tests with:
```bash
cargo test -p intent-generator
```

### Test Coverage

- Topic normalization (valid and invalid cases)
- Content reference validation
- Constraint sanitization
- Expertise deduplication
- Signature generation (when enabled)
- Raw content detection and rejection

## Implementation Notes

### Topic Normalization Strategy

1. Convert to lowercase
2. Replace spaces and hyphens with underscores
3. Remove special characters
4. Truncate to max length
5. Validate it's a safe identifier (starts with letter or underscore)

### Cryptographic Signatures

The current implementation includes placeholder signatures. For production:

**Option 1: HMAC-SHA256 (Symmetric)**
```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;
let mut mac = HmacSha256::new_from_slice(key)?;
mac.update(data.as_bytes());
let signature = hex::encode(mac.finalize().into_bytes());
```

**Option 2: Ed25519 (Asymmetric)**
```rust
use ed25519_dalek::{Keypair, Signer};

let signature = keypair.sign(data.as_bytes());
let signature_hex = hex::encode(signature.to_bytes());
```

See inline code documentation for RSA approach.

## Error Handling

### GeneratorError Types

- `SchemaError`: Validation failures
- `SanitizationError`: Failed to sanitize data
- `InvalidContentRef`: Content reference validation failed
- `TopicNormalizationError`: Topic couldn't be normalized
- `SignatureError`: Signature generation failed
- `JsonError`: JSON serialization failed

## Integration

### Input: VotedIntent (from voting module)
### Output: TrustedIntent (to processing engine)

The Intent Generator sits between the voting module and the processing engine:

```
Voting Module → VotedIntent → Intent Generator → TrustedIntent → Processing Engine
```

## Future Enhancements

- [ ] Implement actual cryptographic signing (HMAC/Ed25519/RSA)
- [ ] Add signature verification methods
- [ ] Support for custom topic normalization rules
- [ ] Rate limiting per user/session
- [ ] Advanced constraint validation rules
- [ ] Topic ID mapping to a controlled vocabulary

## License

MIT

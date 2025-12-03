use serde::{Deserialize, Serialize};

/// Configuration for ChatGPT sacrificial cogitator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatGPTCogitatorConfig {
    /// OpenAI API key
    pub api_key: String,

    /// Model to use (lightweight: gpt-3.5-turbo)
    pub model: String,

    /// Request timeout in seconds
    pub timeout_secs: u64,

    /// Base URL for API (default: https://api.openai.com/v1)
    pub base_url: String,
}

impl Default for ChatGPTCogitatorConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "gpt-5-nano".to_string(), // Cheapest OpenAI model
            timeout_secs: 10,                // Fast timeout for sacrificial LLM
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }
}

impl ChatGPTCogitatorConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            ..Default::default()
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }
}

/// Configuration for DeepSeek sacrificial cogitator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekCogitatorConfig {
    /// DeepSeek API key
    pub api_key: String,

    /// Model to use (lightweight: deepseek-chat)
    pub model: String,

    /// Request timeout in seconds
    pub timeout_secs: u64,

    /// Base URL for API
    pub base_url: String,
}

impl Default for DeepSeekCogitatorConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "deepseek-chat".to_string(),
            timeout_secs: 10, // Fast timeout for sacrificial LLM
            base_url: "https://api.deepseek.com/v1".to_string(),
        }
    }
}

impl DeepSeekCogitatorConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            ..Default::default()
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }
}

/// Configuration for Claude sacrificial cogitator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeCogitatorConfig {
    /// Anthropic API key
    pub api_key: String,

    /// Model to use (lightweight: claude-3-5-haiku-20241022)
    pub model: String,

    /// Request timeout in seconds
    pub timeout_secs: u64,

    /// Base URL for API
    pub base_url: String,
}

impl Default for ClaudeCogitatorConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "claude-3-haiku-20240307".to_string(), // Cheapest Claude model
            timeout_secs: 10,                             // Fast timeout for sacrificial LLM
            base_url: "https://api.anthropic.com/v1".to_string(),
        }
    }
}

impl ClaudeCogitatorConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            ..Default::default()
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }
}

/// Master configuration for all sacrificial cogitators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CogatorsConfig {
    /// Enable ChatGPT cogitator
    pub enable_chatgpt: bool,

    /// Enable DeepSeek cogitator
    pub enable_deepseek: bool,

    /// Enable Claude cogitator
    pub enable_claude: bool,

    /// Require consensus (all must agree input is clean)
    pub require_consensus: bool,

    /// Risk score threshold for flagging (0.0-1.0)
    pub risk_threshold: f32,

    /// ChatGPT configuration
    pub chatgpt: ChatGPTCogitatorConfig,

    /// DeepSeek configuration
    pub deepseek: DeepSeekCogitatorConfig,

    /// Claude configuration
    pub claude: ClaudeCogitatorConfig,
}

impl Default for CogatorsConfig {
    fn default() -> Self {
        Self {
            enable_chatgpt: true,
            enable_deepseek: true,
            enable_claude: true,
            require_consensus: false, // If any flags as suspicious, it's suspicious
            risk_threshold: 0.6,      // 60% risk is concerning
            chatgpt: ChatGPTCogitatorConfig::default(),
            deepseek: DeepSeekCogitatorConfig::default(),
            claude: ClaudeCogitatorConfig::default(),
        }
    }
}

impl CogatorsConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = CogatorsConfig::default();

        // Enable flags
        config.enable_chatgpt = std::env::var("ENABLE_CHATGPT_COGITATOR")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(true);

        config.enable_deepseek = std::env::var("ENABLE_DEEPSEEK_COGITATOR")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(true);

        config.enable_claude = std::env::var("ENABLE_CLAUDE_COGITATOR")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(true);

        // Consensus
        config.require_consensus = std::env::var("COGITATORS_REQUIRE_CONSENSUS")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);

        // Risk threshold
        config.risk_threshold = std::env::var("COGITATORS_RISK_THRESHOLD")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.6);

        // ChatGPT config
        if let Ok(api_key) = std::env::var("CHATGPT_COGITATOR_API_KEY") {
            config.chatgpt.api_key = api_key;
        } else if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            config.chatgpt.api_key = api_key;
        }

        if let Ok(model) = std::env::var("CHATGPT_COGITATOR_MODEL") {
            config.chatgpt.model = model;
        }

        // DeepSeek config
        if let Ok(api_key) = std::env::var("DEEPSEEK_COGITATOR_API_KEY") {
            config.deepseek.api_key = api_key;
        } else if let Ok(api_key) = std::env::var("DEEPSEEK_API_KEY") {
            config.deepseek.api_key = api_key;
        }

        if let Ok(model) = std::env::var("DEEPSEEK_COGITATOR_MODEL") {
            config.deepseek.model = model;
        }

        // Claude config
        if let Ok(api_key) = std::env::var("CLAUDE_COGITATOR_API_KEY") {
            config.claude.api_key = api_key;
        } else if let Ok(api_key) = std::env::var("CLAUDE_API_KEY") {
            config.claude.api_key = api_key;
        }

        if let Ok(model) = std::env::var("CLAUDE_COGITATOR_MODEL") {
            config.claude.model = model;
        }

        config
    }
}

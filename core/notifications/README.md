# Intent Notifications Module

A comprehensive notification system for the Intent Segregation Cybersecurity Architecture, supporting multiple channels including Email, Slack, and Microsoft Teams.

## Features

- **Multi-Channel Notifications**: Send notifications via Email (SMTP), Slack webhooks, and Microsoft Teams webhooks
- **Async/Await Support**: Built on Tokio for efficient asynchronous operations
- **Rich Message Formatting**: Support for HTML emails, Slack attachments, and Teams message cards
- **Approval Requests**: Specialized support for sending approval request notifications
- **Security Alerts**: Send security alerts with priority levels and structured context
- **Type-Safe**: Strongly-typed message structures with serde serialization
- **Flexible Configuration**: TOML-based configuration with environment-specific settings

## Architecture

The module consists of several key components:

- **NotificationService**: Main service coordinating all notification channels
- **Configuration**: Type-safe config structures for each notification channel
- **Message Types**: Structured message formats for different notification types
- **Error Handling**: Comprehensive error types for debugging and monitoring

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
intent-notifications = { path = "../core/notifications" }
tokio = { version = "1.35", features = ["full"] }
```

## Configuration

Create a `notifications.toml` file (see `notifications.example.toml` for a template):

```toml
[notifications.slack]
webhook_url = "https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
username = "Intent Security Bot"

[notifications.teams]
webhook_url = "https://outlook.office.com/webhook/YOUR/WEBHOOK/URL"
theme_color = "0076D7"

[notifications.email]
smtp_server = "smtp.gmail.com"
smtp_port = 587
smtp_user = "your-email@example.com"
smtp_password = "your-app-password"
from_address = "your-email@example.com"
from_name = "Intent Security System"
use_starttls = true
```

## Usage Examples

### Basic Email Notification

```rust
use intent_notifications::{NotificationService, NotificationConfig, EmailMessage};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = NotificationConfig {
        email: Some(EmailConfig {
            smtp_server: "smtp.gmail.com".to_string(),
            smtp_port: 587,
            smtp_user: "your-email@example.com".to_string(),
            smtp_password: "your-password".to_string(),
            from_address: "your-email@example.com".to_string(),
            from_name: "Intent Security".to_string(),
            use_starttls: true,
        }),
        slack: None,
        teams: None,
    };

    let service = NotificationService::new(config);

    // Send email
    let email = EmailMessage {
        to: vec!["admin@example.com".to_string()],
        subject: "Security Alert".to_string(),
        body: "<h1>Alert</h1><p>Suspicious activity detected.</p>".to_string(),
        is_html: true,
        cc: vec![],
    };

    service.send_email(&email).await?;
    Ok(())
}
```

### Slack Notification with Attachments

```rust
use intent_notifications::{
    NotificationService, SlackMessage, SlackAttachment, SlackField
};

async fn send_slack_alert(service: &NotificationService) -> Result<(), Box<dyn std::error::Error>> {
    let attachment = SlackAttachment {
        color: Some("danger".to_string()),
        title: Some("Intent Mismatch Detected".to_string()),
        text: Some("A user intent does not match the provider configuration.".to_string()),
        fields: Some(vec![
            SlackField {
                title: "User".to_string(),
                value: "user@example.com".to_string(),
                short: true,
            },
            SlackField {
                title: "Severity".to_string(),
                value: "High".to_string(),
                short: true,
            },
        ]),
        footer: Some("Intent Security System".to_string()),
        ts: None,
    };

    let message = SlackMessage {
        text: "*Security Alert*".to_string(),
        channel: Some("#security-alerts".to_string()),
        username: None,
        icon_emoji: Some(":warning:".to_string()),
        attachments: Some(vec![attachment]),
    };

    service.send_slack(&message).await?;
    Ok(())
}
```

### Microsoft Teams Notification

```rust
use intent_notifications::{
    NotificationService, TeamsMessage, TeamsSection, TeamsFact
};

async fn send_teams_alert(service: &NotificationService) -> Result<(), Box<dyn std::error::Error>> {
    let facts = vec![
        TeamsFact {
            name: "Alert Type".to_string(),
            value: "Intent Mismatch".to_string(),
        },
        TeamsFact {
            name: "Priority".to_string(),
            value: "High".to_string(),
        },
    ];

    let section = TeamsSection {
        activity_title: Some("Security Alert".to_string()),
        activity_subtitle: Some("Intent validation failed".to_string()),
        facts: Some(facts),
        text: Some("A user's parsed intent does not match allowed operations.".to_string()),
    };

    let message = TeamsMessage {
        title: "Intent Security Alert".to_string(),
        text: "Immediate attention required".to_string(),
        theme_color: Some("FF0000".to_string()),
        sections: Some(vec![section]),
        potential_action: None,
    };

    service.send_teams(&message).await?;
    Ok(())
}
```

### Approval Request (Multi-Channel)

```rust
use intent_notifications::{
    NotificationService, ApprovalRequest, NotificationPriority
};
use chrono::Utc;

async fn request_approval(service: &NotificationService) -> Result<(), Box<dyn std::error::Error>> {
    let request = ApprovalRequest {
        request_id: "req-123456".to_string(),
        title: "Elevated Privilege Request".to_string(),
        description: "User is requesting an action outside normal scope".to_string(),
        requested_by: "user@example.com".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        intent_summary: r#"{
  "action": "delete_resources",
  "scope": "production",
  "resource_count": 50
}"#.to_string(),
        approval_url: Some("https://security.example.com/approvals/req-123456".to_string()),
        priority: NotificationPriority::High,
    };

    // This sends to all configured channels (email, Slack, Teams)
    service.send_approval_request(&request).await?;
    Ok(())
}
```

### Security Alert (Multi-Channel)

```rust
use intent_notifications::{
    NotificationService, Alert, NotificationType, NotificationPriority
};
use chrono::Utc;
use serde_json::json;

async fn send_alert(service: &NotificationService) -> Result<(), Box<dyn std::error::Error>> {
    let alert = Alert {
        alert_id: "alert-789".to_string(),
        title: "Malicious Pattern Detected".to_string(),
        message: "Input contains patterns associated with prompt injection attacks.".to_string(),
        alert_type: NotificationType::Error,
        priority: NotificationPriority::Critical,
        timestamp: Utc::now().to_rfc3339(),
        context: Some(json!({
            "input_hash": "abc123...",
            "pattern": "jailbreak_attempt",
            "confidence": 0.95
        })),
    };

    // This sends to all configured channels
    service.send_alert(&alert).await?;
    Ok(())
}
```

## Message Types

### Email Messages

```rust
pub struct EmailMessage {
    pub to: Vec<String>,        // Recipient addresses
    pub subject: String,         // Email subject
    pub body: String,            // Email body (plain text or HTML)
    pub is_html: bool,           // Whether body is HTML
    pub cc: Vec<String>,         // CC recipients
}
```

### Slack Messages

```rust
pub struct SlackMessage {
    pub text: String,                           // Main message text
    pub channel: Option<String>,                // Channel override
    pub username: Option<String>,               // Username override
    pub icon_emoji: Option<String>,             // Icon emoji
    pub attachments: Option<Vec<SlackAttachment>>, // Rich attachments
}
```

### Teams Messages

```rust
pub struct TeamsMessage {
    pub title: String,                          // Card title
    pub text: String,                           // Card text
    pub theme_color: Option<String>,            // Theme color (hex)
    pub sections: Option<Vec<TeamsSection>>,    // Content sections
    pub potential_action: Option<Vec<TeamsAction>>, // Action buttons
}
```

## Error Handling

The module provides comprehensive error types:

```rust
pub enum NotificationError {
    EmailError(String),           // Email sending failed
    SlackError(String),            // Slack webhook failed
    TeamsError(String),            // Teams webhook failed
    ConfigError(String),           // Configuration error
    HttpError(reqwest::Error),     // HTTP request failed
    SerializationError(serde_json::Error), // JSON serialization failed
    InvalidRecipient(String),      // Invalid email address
    MissingConfig(String),         // Required config missing
}
```

## Integration with Intent Architecture

This module is designed to integrate seamlessly with the Intent Segregation architecture:

1. **Supervision Module**: Request human approval when intent mismatches occur
2. **Comparator Module**: Alert when parsed intents don't match provider configs
3. **Malicious Detector**: Send alerts when potential attacks are detected
4. **Ledger Module**: Notify administrators of audit events

## Security Considerations

- **Credentials**: Never commit `notifications.toml` with real credentials to version control
- **Secrets Management**: Use environment variables or a secrets manager in production
- **SMTP Security**: Always use STARTTLS or SSL/TLS for email
- **Webhook URLs**: Treat webhook URLs as secrets
- **Rate Limiting**: Consider implementing rate limiting for high-volume scenarios
- **App Passwords**: Use app-specific passwords for email services (e.g., Gmail App Passwords)

## Testing

Run tests with:

```bash
cargo test -p intent-notifications
```

For integration tests with real services, set up test webhooks and email accounts, then run:

```bash
cargo test -p intent-notifications -- --ignored
```

## Dependencies

- **tokio**: Async runtime
- **lettre**: Email sending (SMTP)
- **reqwest**: HTTP client for webhooks
- **serde/serde_json**: Serialization
- **thiserror**: Error handling
- **tracing**: Logging
- **regex**: HTML stripping for plain text emails

## License

MIT

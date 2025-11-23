use crate::config::{EmailConfig, NotificationConfig};
use crate::error::{NotificationError, Result};
use crate::types::{Alert, ApprovalRequest, EmailMessage, SlackMessage, TeamsMessage};
use lettre::{
    message::{header::ContentType, Message, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};
use reqwest::Client;
use serde_json::json;
use tracing::{error, info, warn};

/// Notification service for sending emails, Slack, and Teams messages
#[derive(Debug, Clone)]
pub struct NotificationService {
    config: NotificationConfig,
    http_client: Client,
}

impl NotificationService {
    /// Create a new notification service with the given configuration
    pub fn new(config: NotificationConfig) -> Self {
        Self {
            config,
            http_client: Client::new(),
        }
    }

    /// Send an email message
    ///
    /// # Arguments
    /// * `message` - Email message to send
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Example
    /// ```no_run
    /// use intent_notifications::{NotificationService, EmailMessage, NotificationConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = NotificationConfig::default();
    /// let service = NotificationService::new(config);
    ///
    /// let email = EmailMessage {
    ///     to: vec!["user@example.com".to_string()],
    ///     subject: "Test Email".to_string(),
    ///     body: "This is a test email.".to_string(),
    ///     is_html: false,
    ///     cc: vec![],
    /// };
    ///
    /// service.send_email(&email).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_email(&self, message: &EmailMessage) -> Result<()> {
        let email_config = self.config.email.as_ref().ok_or_else(|| {
            NotificationError::MissingConfig("Email configuration not provided".to_string())
        })?;

        info!("Sending email to {:?}", message.to);

        // Validate recipients
        if message.to.is_empty() {
            return Err(NotificationError::InvalidRecipient(
                "No recipients specified".to_string(),
            ));
        }

        // Build the email
        let mut email_builder = Message::builder().from(
            format!("{} <{}>", email_config.from_name, email_config.from_address)
                .parse()
                .map_err(|e| {
                    NotificationError::EmailError(format!("Invalid from address: {}", e))
                })?,
        );

        // Add recipients
        for recipient in &message.to {
            email_builder = email_builder.to(recipient.parse().map_err(|e| {
                NotificationError::InvalidRecipient(format!(
                    "Invalid recipient {}: {}",
                    recipient, e
                ))
            })?);
        }

        // Add CC recipients
        for cc in &message.cc {
            email_builder = email_builder.cc(cc.parse().map_err(|e| {
                NotificationError::InvalidRecipient(format!("Invalid CC {}: {}", cc, e))
            })?);
        }

        // Set subject
        email_builder = email_builder.subject(&message.subject);

        // Build message body
        let email = if message.is_html {
            email_builder
                .multipart(
                    MultiPart::alternative()
                        .singlepart(
                            SinglePart::builder()
                                .header(ContentType::TEXT_PLAIN)
                                .body(strip_html(&message.body)),
                        )
                        .singlepart(
                            SinglePart::builder()
                                .header(ContentType::TEXT_HTML)
                                .body(message.body.clone()),
                        ),
                )
                .map_err(|e| {
                    NotificationError::EmailError(format!("Failed to build email: {}", e))
                })?
        } else {
            email_builder.body(message.body.clone()).map_err(|e| {
                NotificationError::EmailError(format!("Failed to build email: {}", e))
            })?
        };

        // Send the email
        self.send_smtp_email(email, email_config).await?;

        info!("Email sent successfully to {:?}", message.to);
        Ok(())
    }

    /// Internal method to send email via SMTP
    async fn send_smtp_email(&self, email: Message, config: &EmailConfig) -> Result<()> {
        let credentials = Credentials::new(config.smtp_user.clone(), config.smtp_password.clone());

        let mailer = if config.use_starttls {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)
                .map_err(|e| {
                    NotificationError::EmailError(format!("SMTP connection failed: {}", e))
                })?
                .credentials(credentials)
                .port(config.smtp_port)
                .build()
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_server)
                .map_err(|e| {
                    NotificationError::EmailError(format!("SMTP connection failed: {}", e))
                })?
                .credentials(credentials)
                .port(config.smtp_port)
                .build()
        };

        mailer
            .send(email)
            .await
            .map_err(|e| NotificationError::EmailError(format!("Failed to send email: {}", e)))?;

        Ok(())
    }

    /// Send a Slack message via webhook
    ///
    /// # Arguments
    /// * `message` - Slack message to send
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Example
    /// ```no_run
    /// use intent_notifications::{NotificationService, SlackMessage, NotificationConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = NotificationConfig::default();
    /// let service = NotificationService::new(config);
    ///
    /// let slack_msg = SlackMessage {
    ///     text: "Hello from Intent Security!".to_string(),
    ///     channel: None,
    ///     username: None,
    ///     icon_emoji: Some(":shield:".to_string()),
    ///     attachments: None,
    /// };
    ///
    /// service.send_slack(&slack_msg).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_slack(&self, message: &SlackMessage) -> Result<()> {
        let slack_config = self.config.slack.as_ref().ok_or_else(|| {
            NotificationError::MissingConfig("Slack configuration not provided".to_string())
        })?;

        info!("Sending Slack message");

        // Build the payload
        let mut payload = json!({
            "text": message.text,
        });

        // Add optional fields
        if let Some(channel) = &message.channel {
            payload["channel"] = json!(channel);
        }

        if let Some(username) = &message.username {
            payload["username"] = json!(username);
        } else {
            payload["username"] = json!(slack_config.username);
        }

        if let Some(icon_emoji) = &message.icon_emoji {
            payload["icon_emoji"] = json!(icon_emoji);
        }

        if let Some(attachments) = &message.attachments {
            payload["attachments"] = serde_json::to_value(attachments)?;
        }

        // Send the webhook request
        let response = self
            .http_client
            .post(&slack_config.webhook_url)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                "Slack webhook failed with status {}: {}",
                status, error_text
            );
            return Err(NotificationError::SlackError(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        info!("Slack message sent successfully");
        Ok(())
    }

    /// Send a Microsoft Teams message via webhook
    ///
    /// # Arguments
    /// * `message` - Teams message to send
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Example
    /// ```no_run
    /// use intent_notifications::{NotificationService, TeamsMessage, NotificationConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = NotificationConfig::default();
    /// let service = NotificationService::new(config);
    ///
    /// let teams_msg = TeamsMessage {
    ///     title: "Security Alert".to_string(),
    ///     text: "Intent mismatch detected!".to_string(),
    ///     theme_color: Some("FF0000".to_string()),
    ///     sections: None,
    ///     potential_action: None,
    /// };
    ///
    /// service.send_teams(&teams_msg).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_teams(&self, message: &TeamsMessage) -> Result<()> {
        let teams_config = self.config.teams.as_ref().ok_or_else(|| {
            NotificationError::MissingConfig("Teams configuration not provided".to_string())
        })?;

        info!("Sending Teams message");

        // Build the MessageCard payload
        let mut payload = json!({
            "@type": "MessageCard",
            "@context": "https://schema.org/extensions",
            "title": message.title,
            "text": message.text,
        });

        // Add theme color
        if let Some(color) = &message.theme_color {
            payload["themeColor"] = json!(color);
        } else {
            payload["themeColor"] = json!(teams_config.theme_color);
        }

        // Add sections
        if let Some(sections) = &message.sections {
            payload["sections"] = serde_json::to_value(sections)?;
        }

        // Add potential actions
        if let Some(actions) = &message.potential_action {
            payload["potentialAction"] = serde_json::to_value(actions)?;
        }

        // Send the webhook request
        let response = self
            .http_client
            .post(&teams_config.webhook_url)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                "Teams webhook failed with status {}: {}",
                status, error_text
            );
            return Err(NotificationError::TeamsError(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        info!("Teams message sent successfully");
        Ok(())
    }

    /// Send an approval request notification to all configured channels
    ///
    /// # Arguments
    /// * `request` - Approval request details
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn send_approval_request(&self, request: &ApprovalRequest) -> Result<()> {
        info!("Sending approval request: {}", request.request_id);

        let mut errors = Vec::new();

        // Send to email if configured
        if self.config.has_email() {
            if let Err(e) = self.send_approval_email(request).await {
                warn!("Failed to send approval email: {}", e);
                errors.push(format!("Email: {}", e));
            }
        }

        // Send to Slack if configured
        if self.config.has_slack() {
            if let Err(e) = self.send_approval_slack(request).await {
                warn!("Failed to send approval to Slack: {}", e);
                errors.push(format!("Slack: {}", e));
            }
        }

        // Send to Teams if configured
        if self.config.has_teams() {
            if let Err(e) = self.send_approval_teams(request).await {
                warn!("Failed to send approval to Teams: {}", e);
                errors.push(format!("Teams: {}", e));
            }
        }

        if !errors.is_empty() {
            return Err(NotificationError::ConfigError(format!(
                "Some notifications failed: {}",
                errors.join(", ")
            )));
        }

        Ok(())
    }

    /// Send an alert to all configured channels
    ///
    /// # Arguments
    /// * `alert` - Alert details
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn send_alert(&self, alert: &Alert) -> Result<()> {
        info!("Sending alert: {}", alert.alert_id);

        let mut errors = Vec::new();

        // Send to email if configured
        if self.config.has_email() {
            if let Err(e) = self.send_alert_email(alert).await {
                warn!("Failed to send alert email: {}", e);
                errors.push(format!("Email: {}", e));
            }
        }

        // Send to Slack if configured
        if self.config.has_slack() {
            if let Err(e) = self.send_alert_slack(alert).await {
                warn!("Failed to send alert to Slack: {}", e);
                errors.push(format!("Slack: {}", e));
            }
        }

        // Send to Teams if configured
        if self.config.has_teams() {
            if let Err(e) = self.send_alert_teams(alert).await {
                warn!("Failed to send alert to Teams: {}", e);
                errors.push(format!("Teams: {}", e));
            }
        }

        if !errors.is_empty() {
            return Err(NotificationError::ConfigError(format!(
                "Some notifications failed: {}",
                errors.join(", ")
            )));
        }

        Ok(())
    }

    // Helper methods for approval requests

    async fn send_approval_email(&self, request: &ApprovalRequest) -> Result<()> {
        let body = format!(
            r#"<h2>Approval Request</h2>
<p><strong>Request ID:</strong> {}</p>
<p><strong>Title:</strong> {}</p>
<p><strong>Requested by:</strong> {}</p>
<p><strong>Time:</strong> {}</p>
<p><strong>Priority:</strong> {:?}</p>

<h3>Description</h3>
<p>{}</p>

<h3>Intent Summary</h3>
<pre>{}</pre>

{}
"#,
            request.request_id,
            request.title,
            request.requested_by,
            request.timestamp,
            request.priority,
            request.description,
            request.intent_summary,
            request
                .approval_url
                .as_ref()
                .map(|url| format!(
                    "<p><a href=\"{}\">Click here to review and approve</a></p>",
                    url
                ))
                .unwrap_or_default()
        );

        let email = EmailMessage {
            to: vec![], // This should be configured with admin emails
            subject: format!("[APPROVAL REQUIRED] {}", request.title),
            body,
            is_html: true,
            cc: vec![],
        };

        self.send_email(&email).await
    }

    async fn send_approval_slack(&self, request: &ApprovalRequest) -> Result<()> {
        use crate::types::{SlackAttachment, SlackField};

        let color = match request.priority {
            crate::types::NotificationPriority::Critical => "danger",
            crate::types::NotificationPriority::High => "warning",
            _ => "good",
        };

        let mut fields = vec![
            SlackField {
                title: "Request ID".to_string(),
                value: request.request_id.clone(),
                short: true,
            },
            SlackField {
                title: "Requested By".to_string(),
                value: request.requested_by.clone(),
                short: true,
            },
            SlackField {
                title: "Priority".to_string(),
                value: format!("{:?}", request.priority),
                short: true,
            },
            SlackField {
                title: "Intent Summary".to_string(),
                value: format!("```{}```", request.intent_summary),
                short: false,
            },
        ];

        if let Some(url) = &request.approval_url {
            fields.push(SlackField {
                title: "Action".to_string(),
                value: format!("<{}|Review and Approve>", url),
                short: false,
            });
        }

        let message = SlackMessage {
            text: format!("*Approval Required:* {}", request.title),
            channel: None,
            username: None,
            icon_emoji: Some(":warning:".to_string()),
            attachments: Some(vec![SlackAttachment {
                color: Some(color.to_string()),
                title: Some(request.description.clone()),
                text: None,
                fields: Some(fields),
                footer: Some("Intent Security System".to_string()),
                ts: None,
            }]),
        };

        self.send_slack(&message).await
    }

    async fn send_approval_teams(&self, request: &ApprovalRequest) -> Result<()> {
        use crate::types::{TeamsAction, TeamsActionTarget, TeamsFact, TeamsSection};

        let color = match request.priority {
            crate::types::NotificationPriority::Critical => "FF0000",
            crate::types::NotificationPriority::High => "FFA500",
            _ => "00FF00",
        };

        let facts = vec![
            TeamsFact {
                name: "Request ID".to_string(),
                value: request.request_id.clone(),
            },
            TeamsFact {
                name: "Requested By".to_string(),
                value: request.requested_by.clone(),
            },
            TeamsFact {
                name: "Priority".to_string(),
                value: format!("{:?}", request.priority),
            },
            TeamsFact {
                name: "Time".to_string(),
                value: request.timestamp.clone(),
            },
        ];

        let sections = vec![TeamsSection {
            activity_title: Some(request.title.clone()),
            activity_subtitle: Some(request.description.clone()),
            facts: Some(facts),
            text: Some(format!(
                "**Intent Summary:**\n```\n{}\n```",
                request.intent_summary
            )),
        }];

        let actions = request.approval_url.as_ref().map(|url| {
            vec![TeamsAction {
                action_type: "OpenUri".to_string(),
                name: "Review and Approve".to_string(),
                targets: Some(vec![TeamsActionTarget {
                    os: "default".to_string(),
                    uri: url.clone(),
                }]),
            }]
        });

        let message = TeamsMessage {
            title: "Approval Request".to_string(),
            text: format!("An approval request requires your attention."),
            theme_color: Some(color.to_string()),
            sections: Some(sections),
            potential_action: actions,
        };

        self.send_teams(&message).await
    }

    // Helper methods for alerts

    async fn send_alert_email(&self, alert: &Alert) -> Result<()> {
        let body = format!(
            r#"<h2>Security Alert</h2>
<p><strong>Alert ID:</strong> {}</p>
<p><strong>Type:</strong> {:?}</p>
<p><strong>Priority:</strong> {:?}</p>
<p><strong>Time:</strong> {}</p>

<h3>{}</h3>
<p>{}</p>

{}
"#,
            alert.alert_id,
            alert.alert_type,
            alert.priority,
            alert.timestamp,
            alert.title,
            alert.message,
            alert
                .context
                .as_ref()
                .map(|ctx| format!(
                    "<h3>Context</h3><pre>{}</pre>",
                    serde_json::to_string_pretty(ctx).unwrap_or_default()
                ))
                .unwrap_or_default()
        );

        let email = EmailMessage {
            to: vec![], // This should be configured with admin emails
            subject: format!("[ALERT] {}", alert.title),
            body,
            is_html: true,
            cc: vec![],
        };

        self.send_email(&email).await
    }

    async fn send_alert_slack(&self, alert: &Alert) -> Result<()> {
        use crate::types::{SlackAttachment, SlackField};

        let color = match alert.priority {
            crate::types::NotificationPriority::Critical => "danger",
            crate::types::NotificationPriority::High => "warning",
            _ => "#439FE0",
        };

        let fields = vec![
            SlackField {
                title: "Alert ID".to_string(),
                value: alert.alert_id.clone(),
                short: true,
            },
            SlackField {
                title: "Type".to_string(),
                value: format!("{:?}", alert.alert_type),
                short: true,
            },
            SlackField {
                title: "Priority".to_string(),
                value: format!("{:?}", alert.priority),
                short: true,
            },
            SlackField {
                title: "Time".to_string(),
                value: alert.timestamp.clone(),
                short: true,
            },
        ];

        let message = SlackMessage {
            text: format!("*Alert:* {}", alert.title),
            channel: None,
            username: None,
            icon_emoji: Some(":rotating_light:".to_string()),
            attachments: Some(vec![SlackAttachment {
                color: Some(color.to_string()),
                title: None,
                text: Some(alert.message.clone()),
                fields: Some(fields),
                footer: Some("Intent Security System".to_string()),
                ts: None,
            }]),
        };

        self.send_slack(&message).await
    }

    async fn send_alert_teams(&self, alert: &Alert) -> Result<()> {
        use crate::types::{TeamsFact, TeamsSection};

        let color = match alert.priority {
            crate::types::NotificationPriority::Critical => "FF0000",
            crate::types::NotificationPriority::High => "FFA500",
            _ => "0076D7",
        };

        let facts = vec![
            TeamsFact {
                name: "Alert ID".to_string(),
                value: alert.alert_id.clone(),
            },
            TeamsFact {
                name: "Type".to_string(),
                value: format!("{:?}", alert.alert_type),
            },
            TeamsFact {
                name: "Priority".to_string(),
                value: format!("{:?}", alert.priority),
            },
            TeamsFact {
                name: "Time".to_string(),
                value: alert.timestamp.clone(),
            },
        ];

        let sections = vec![TeamsSection {
            activity_title: Some(alert.title.clone()),
            activity_subtitle: None,
            facts: Some(facts),
            text: Some(alert.message.clone()),
        }];

        let message = TeamsMessage {
            title: "Security Alert".to_string(),
            text: "A security alert has been triggered.".to_string(),
            theme_color: Some(color.to_string()),
            sections: Some(sections),
            potential_action: None,
        };

        self.send_teams(&message).await
    }
}

/// Strip HTML tags for plain text email fallback
fn strip_html(html: &str) -> String {
    // Simple HTML stripping - in production, use a proper HTML parser
    let re = regex::Regex::new(r"<[^>]*>").unwrap();
    re.replace_all(html, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_html() {
        let html = "<h1>Hello</h1><p>World</p>";
        assert_eq!(strip_html(html), "HelloWorld");
    }
}

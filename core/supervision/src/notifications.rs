use crate::error::{Result, SupervisionError};
use crate::models::{ApprovalRequest, NotificationChannel, SupervisionConfig};
use tracing::{debug, error, info};

/// Trait for sending approval notifications
#[async_trait::async_trait]
pub trait NotificationService: Send + Sync {
    /// Send a notification for a new approval request
    async fn notify_approval_request(&self, request: &ApprovalRequest) -> Result<()>;

    /// Send a notification that a request has been approved
    async fn notify_approved(&self, request_id: &str, approver: &str) -> Result<()>;

    /// Send a notification that a request has been denied
    async fn notify_denied(
        &self,
        request_id: &str,
        approver: &str,
        reason: Option<&str>,
    ) -> Result<()>;

    /// Send a notification that a request has expired
    async fn notify_expired(&self, request_id: &str) -> Result<()>;
}

/// Multi-channel notification service
pub struct MultiChannelNotifier {
    #[allow(dead_code)]
    config: SupervisionConfig,
    email_client: Option<EmailNotifier>,
    slack_client: Option<SlackNotifier>,
    teams_client: Option<TeamsNotifier>,
}

impl MultiChannelNotifier {
    /// Create a new multi-channel notifier
    pub fn new(config: SupervisionConfig) -> Self {
        let email_client = if config
            .notification_channels
            .contains(&NotificationChannel::Email)
        {
            Some(EmailNotifier::new(config.notification_emails.clone()))
        } else {
            None
        };

        let slack_client = if config
            .notification_channels
            .contains(&NotificationChannel::Slack)
        {
            config
                .slack_webhook_url
                .as_ref()
                .map(|url| SlackNotifier::new(url.clone()))
        } else {
            None
        };

        let teams_client = if config
            .notification_channels
            .contains(&NotificationChannel::Teams)
        {
            config
                .teams_webhook_url
                .as_ref()
                .map(|url| TeamsNotifier::new(url.clone()))
        } else {
            None
        };

        Self {
            config,
            email_client,
            slack_client,
            teams_client,
        }
    }

    /// Format the approval request as a human-readable message
    fn format_approval_message(&self, request: &ApprovalRequest) -> String {
        let mut message = format!(
            "🔔 New Approval Request\n\n\
             Request ID: {}\n\
             Risk Level: {:?}\n\
             Action: {}\n\
             Created: {}\n\n",
            request.id, request.risk_level, request.intent.action, request.created_at
        );

        if let Some(topic) = &request.intent.topic {
            message.push_str(&format!("Topic: {}\n", topic));
        }

        message.push_str("\nReasons for Approval:\n");
        for reason in &request.reasons {
            message.push_str(&format!(
                "  - {}: {}\n",
                reason.reason_type, reason.description
            ));
        }

        message.push_str(&format!("\nRaw Input: {}\n", request.raw_input));

        if let Some(expires_at) = request.expires_at {
            message.push_str(&format!("\nExpires: {}\n", expires_at));
        }

        message
    }
}

#[async_trait::async_trait]
impl NotificationService for MultiChannelNotifier {
    async fn notify_approval_request(&self, request: &ApprovalRequest) -> Result<()> {
        let message = self.format_approval_message(request);
        let mut errors = Vec::new();

        // Send to all configured channels
        if let Some(email) = &self.email_client {
            if let Err(e) = email.send_approval_request(&message).await {
                error!(error = %e, "Failed to send email notification");
                errors.push(format!("Email: {}", e));
            }
        }

        if let Some(slack) = &self.slack_client {
            if let Err(e) = slack.send_approval_request(&message).await {
                error!(error = %e, "Failed to send Slack notification");
                errors.push(format!("Slack: {}", e));
            }
        }

        if let Some(teams) = &self.teams_client {
            if let Err(e) = teams.send_approval_request(&message).await {
                error!(error = %e, "Failed to send Teams notification");
                errors.push(format!("Teams: {}", e));
            }
        }

        if !errors.is_empty() {
            return Err(SupervisionError::NotificationFailed(errors.join("; ")));
        }

        info!(request_id = %request.id, "Approval request notification sent");
        Ok(())
    }

    async fn notify_approved(&self, request_id: &str, approver: &str) -> Result<()> {
        let message = format!(
            "✅ Approval Request {} has been APPROVED by {}",
            request_id, approver
        );

        debug!(request_id, approver, "Sending approval notification");

        // Send to all configured channels (simplified for status updates)
        if let Some(email) = &self.email_client {
            let _ = email.send_status_update(&message).await;
        }
        if let Some(slack) = &self.slack_client {
            let _ = slack.send_status_update(&message).await;
        }
        if let Some(teams) = &self.teams_client {
            let _ = teams.send_status_update(&message).await;
        }

        Ok(())
    }

    async fn notify_denied(
        &self,
        request_id: &str,
        approver: &str,
        reason: Option<&str>,
    ) -> Result<()> {
        let message = if let Some(reason) = reason {
            format!(
                "❌ Approval Request {} has been DENIED by {}\nReason: {}",
                request_id, approver, reason
            )
        } else {
            format!(
                "❌ Approval Request {} has been DENIED by {}",
                request_id, approver
            )
        };

        debug!(request_id, approver, "Sending denial notification");

        // Send to all configured channels
        if let Some(email) = &self.email_client {
            let _ = email.send_status_update(&message).await;
        }
        if let Some(slack) = &self.slack_client {
            let _ = slack.send_status_update(&message).await;
        }
        if let Some(teams) = &self.teams_client {
            let _ = teams.send_status_update(&message).await;
        }

        Ok(())
    }

    async fn notify_expired(&self, request_id: &str) -> Result<()> {
        let message = format!("⏰ Approval Request {} has EXPIRED", request_id);

        debug!(request_id, "Sending expiration notification");

        // Send to all configured channels
        if let Some(email) = &self.email_client {
            let _ = email.send_status_update(&message).await;
        }
        if let Some(slack) = &self.slack_client {
            let _ = slack.send_status_update(&message).await;
        }
        if let Some(teams) = &self.teams_client {
            let _ = teams.send_status_update(&message).await;
        }

        Ok(())
    }
}

/// Email notification client
pub struct EmailNotifier {
    recipients: Vec<String>,
}

impl EmailNotifier {
    pub fn new(recipients: Vec<String>) -> Self {
        Self { recipients }
    }

    async fn send_approval_request(&self, message: &str) -> Result<()> {
        // TODO: Integrate with intent-notifications crate when it's implemented
        // For now, log the notification
        info!(
            recipients = ?self.recipients,
            "Email notification (not yet implemented): {}",
            message
        );
        Ok(())
    }

    async fn send_status_update(&self, message: &str) -> Result<()> {
        info!(
            recipients = ?self.recipients,
            "Email status update: {}",
            message
        );
        Ok(())
    }
}

/// Slack notification client
pub struct SlackNotifier {
    webhook_url: String,
}

impl SlackNotifier {
    pub fn new(webhook_url: String) -> Self {
        Self { webhook_url }
    }

    async fn send_approval_request(&self, message: &str) -> Result<()> {
        self.send_message(message).await
    }

    async fn send_status_update(&self, message: &str) -> Result<()> {
        self.send_message(message).await
    }

    async fn send_message(&self, message: &str) -> Result<()> {
        // TODO: Use reqwest to send to Slack webhook
        // For now, log the notification
        info!(
            webhook_url = %self.webhook_url,
            "Slack notification (not yet implemented): {}",
            message
        );
        Ok(())
    }
}

/// Microsoft Teams notification client
pub struct TeamsNotifier {
    webhook_url: String,
}

impl TeamsNotifier {
    pub fn new(webhook_url: String) -> Self {
        Self { webhook_url }
    }

    async fn send_approval_request(&self, message: &str) -> Result<()> {
        self.send_message(message).await
    }

    async fn send_status_update(&self, message: &str) -> Result<()> {
        self.send_message(message).await
    }

    async fn send_message(&self, message: &str) -> Result<()> {
        // TODO: Use reqwest to send to Teams webhook
        // For now, log the notification
        info!(
            webhook_url = %self.webhook_url,
            "Teams notification (not yet implemented): {}",
            message
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ApprovalReason, Intent, RiskLevel};
    use std::collections::HashMap;

    fn create_test_request() -> ApprovalRequest {
        let intent = Intent {
            action: "test_action".to_string(),
            topic: Some("test_topic".to_string()),
            parameters: HashMap::new(),
            content_refs: vec![],
        };

        let reason = ApprovalReason {
            reason_type: "test_reason".to_string(),
            description: "Test description".to_string(),
            metadata: HashMap::new(),
        };

        ApprovalRequest::new(
            intent,
            vec![reason],
            RiskLevel::Medium,
            "test input".to_string(),
            vec![],
        )
    }

    #[tokio::test]
    async fn test_multi_channel_notifier_creation() {
        let config = SupervisionConfig::default();
        let notifier = MultiChannelNotifier::new(config);
        // Just ensure it creates successfully
        assert!(notifier.email_client.is_some());
    }

    #[tokio::test]
    async fn test_format_approval_message() {
        let config = SupervisionConfig::default();
        let notifier = MultiChannelNotifier::new(config);
        let request = create_test_request();
        let message = notifier.format_approval_message(&request);
        assert!(message.contains("New Approval Request"));
        assert!(message.contains("test_action"));
    }
}

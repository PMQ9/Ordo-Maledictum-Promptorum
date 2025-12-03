//! Notification batching for cost optimization
//!
//! Batches multiple notifications together to reduce API calls:
//! - Collects notifications in a queue for 30 seconds
//! - Combines multiple notifications into single messages
//! - Reduces redundant approval/alert notifications to users
//!
//! This is especially useful for:
//! - Multiple approvals needed at same time
//! - Multiple security alerts (combine into single message)
//! - Burst of status updates (aggregate into summary)

use crate::types::{Alert, ApprovalRequest, EmailMessage, SlackMessage, TeamsMessage};
use chrono::Utc;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

/// Notification batch types
#[derive(Debug, Clone)]
pub enum BatchedNotification {
    ApprovalRequest(ApprovalRequest),
    Alert(Alert),
    Email(EmailMessage),
}

/// Notification batching queue
pub struct NotificationBatcher {
    /// Queue of pending notifications
    queue: Arc<Mutex<VecDeque<BatchedNotification>>>,
    /// Batch window duration (30 seconds default)
    batch_window_ms: u64,
}

impl NotificationBatcher {
    /// Create a new notification batcher with 30-second batch window
    pub fn new(batch_window_ms: u64) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            batch_window_ms,
        }
    }

    /// Default batcher with 30-second batching window
    pub fn default_30s() -> Self {
        Self::new(30000)
    }

    /// Queue a notification for batching
    pub async fn queue_approval_request(&self, request: ApprovalRequest) {
        let mut queue = self.queue.lock().await;
        queue.push_back(BatchedNotification::ApprovalRequest(request));
        tracing::debug!(
            "Approval request queued for batching. Queue size: {}",
            queue.len()
        );
    }

    /// Queue an alert for batching
    pub async fn queue_alert(&self, alert: Alert) {
        let mut queue = self.queue.lock().await;
        queue.push_back(BatchedNotification::Alert(alert));
        tracing::debug!("Alert queued for batching. Queue size: {}", queue.len());
    }

    /// Queue an email for batching
    pub async fn queue_email(&self, email: EmailMessage) {
        let mut queue = self.queue.lock().await;
        queue.push_back(BatchedNotification::Email(email));
        tracing::debug!("Email queued for batching. Queue size: {}", queue.len());
    }

    /// Drain all pending notifications from the queue
    pub async fn drain_all(&self) -> Vec<BatchedNotification> {
        let mut queue = self.queue.lock().await;
        let notifications: Vec<_> = queue.drain(..).collect();

        if !notifications.is_empty() {
            tracing::info!(
                "Drained {} notifications from batch queue",
                notifications.len()
            );
        }

        notifications
    }

    /// Get current queue size
    pub async fn queue_size(&self) -> usize {
        let queue = self.queue.lock().await;
        queue.len()
    }

    /// Start background batching with periodic drain
    ///
    /// This starts a background task that drains the queue every batch_window_ms
    /// and returns a shutdown handle.
    pub fn start_background_batcher(
        self: Arc<Self>,
        drain_callback: impl Fn(Vec<BatchedNotification>) + Send + 'static,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(self.batch_window_ms));

            loop {
                interval.tick().await;

                let notifications = self.drain_all().await;
                if !notifications.is_empty() {
                    drain_callback(notifications);
                }
            }
        })
    }
}

/// Helper to combine multiple alerts into a single Slack message
pub fn combine_alerts_to_slack(alerts: &[Alert]) -> Option<SlackMessage> {
    if alerts.is_empty() {
        return None;
    }

    // Determine severity
    let has_critical = alerts.iter().any(|a| {
        matches!(
            a.priority,
            crate::types::NotificationPriority::Critical | crate::types::NotificationPriority::High
        )
    });

    let color = if has_critical {
        "#d32f2f".to_string() // Red
    } else {
        "#ff9800".to_string() // Orange
    };

    // Build alert text
    let mut text = format!(
        "Security Alert Summary - {} alerts at {}",
        alerts.len(),
        Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );

    // Add individual alert details
    for (i, alert) in alerts.iter().enumerate() {
        text.push_str(&format!(
            "\n\n{}. {} ({})\n   {}",
            i + 1,
            alert.title,
            format!("{:?}", alert.priority),
            alert.message
        ));
    }

    Some(SlackMessage {
        text: format!("Alert Batch: {} security alerts", alerts.len()),
        channel: None,
        username: Some("Intent Security Alerts".to_string()),
        icon_emoji: Some(":warning:".to_string()),
        attachments: Some(vec![crate::types::SlackAttachment {
            color: Some(color),
            title: Some("Batched Security Alerts".to_string()),
            text: Some(text),
            fields: None,
            footer: Some("Intent Security System".to_string()),
            ts: Some(Utc::now().timestamp()),
        }]),
    })
}

/// Helper to combine multiple approval requests into a single Teams message
pub fn combine_approvals_to_teams(approvals: &[ApprovalRequest]) -> Option<TeamsMessage> {
    if approvals.is_empty() {
        return None;
    }

    let mut facts = Vec::new();
    for (i, approval) in approvals.iter().enumerate() {
        facts.push(crate::types::TeamsFact {
            name: format!("Request {}", i + 1),
            value: approval.title.clone(),
        });
    }

    Some(TeamsMessage {
        title: format!("Batched Approvals: {} requests pending", approvals.len()),
        text: format!("{} approval request(s) require attention", approvals.len()),
        theme_color: Some("ff6b6b".to_string()),
        sections: Some(vec![crate::types::TeamsSection {
            activity_title: Some("Pending Approvals".to_string()),
            activity_subtitle: None,
            facts: Some(facts),
            text: None,
        }]),
        potential_action: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{NotificationPriority, NotificationType};

    #[tokio::test]
    async fn test_notification_queueing() {
        let batcher = NotificationBatcher::default_30s();

        let alert = Alert {
            alert_id: "test1".to_string(),
            title: "Test Alert".to_string(),
            message: "Test message".to_string(),
            alert_type: NotificationType::Alert,
            priority: NotificationPriority::High,
            timestamp: Utc::now().to_rfc3339(),
            context: None,
        };

        batcher.queue_alert(alert).await;
        assert_eq!(batcher.queue_size().await, 1);

        let drained = batcher.drain_all().await;
        assert_eq!(drained.len(), 1);
        assert_eq!(batcher.queue_size().await, 0);
    }

    #[test]
    fn test_combine_alerts() {
        let alerts = vec![
            Alert {
                alert_id: "1".to_string(),
                title: "SQL Injection Attempt".to_string(),
                message: "Detected malicious pattern".to_string(),
                alert_type: NotificationType::Alert,
                priority: NotificationPriority::High,
                timestamp: Utc::now().to_rfc3339(),
                context: None,
            },
            Alert {
                alert_id: "2".to_string(),
                title: "Prompt Injection".to_string(),
                message: "Suspicious input detected".to_string(),
                alert_type: NotificationType::Alert,
                priority: NotificationPriority::Critical,
                timestamp: Utc::now().to_rfc3339(),
                context: None,
            },
        ];

        let slack_msg = combine_alerts_to_slack(&alerts);
        assert!(slack_msg.is_some());

        let msg = slack_msg.unwrap();
        assert!(msg.text.contains("2 alerts"));
        assert!(msg.attachments.is_some());
    }
}

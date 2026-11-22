use crate::domain::value_objects::{TicketId, UserId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketMessage {
    pub id: String,
    pub ticket_id: TicketId,
    pub sender_id: UserId,
    pub sender_role: String, // "user" or "admin"
    pub message: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    pub id: TicketId,
    pub user_id: UserId,
    pub subject: String,
    pub status: String, // "open", "responded", "closed"
    pub messages: Vec<TicketMessage>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Ticket {
    pub fn new(user_id: UserId, subject: String, initial_message: String) -> Self {
        let now = Utc::now();
        let ticket_id = TicketId::new();
        let message = TicketMessage {
            id: uuid::Uuid::new_v4().to_string(),
            ticket_id: ticket_id.clone(),
            sender_id: user_id.clone(),
            sender_role: "user".to_string(),
            message: initial_message,
            created_at: now,
        };

        Self {
            id: ticket_id,
            user_id,
            subject,
            status: "open".to_string(),
            messages: vec![message],
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_reply(&mut self, sender_id: UserId, role: String, message: String) {
        let now = Utc::now();
        self.messages.push(TicketMessage {
            id: uuid::Uuid::new_v4().to_string(),
            ticket_id: self.id.clone(),
            sender_id,
            sender_role: role.clone(),
            message,
            created_at: now,
        });
        self.status = if role == "admin" {
            "responded".to_string()
        } else {
            "open".to_string()
        };
        self.updated_at = now;
    }
}

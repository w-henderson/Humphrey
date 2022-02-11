use humphrey_ws::Message;

pub struct BroadcastMessage {
    pub message: String,
    pub sender_id: usize,
    pub sender_name: Option<String>,
}

impl BroadcastMessage {
    pub fn serialise(&self) -> Message {
        let mut bytes = Vec::with_capacity(
            self.message.len() + self.sender_name.as_ref().map(|s| s.len()).unwrap_or(0) + 24,
        );

        bytes.extend_from_slice(&self.message.len().to_be_bytes());
        bytes.extend_from_slice(self.message.as_bytes());
        bytes.extend_from_slice(&self.sender_id.to_be_bytes());

        if let Some(sender) = &self.sender_name {
            bytes.extend_from_slice(&sender.len().to_be_bytes());
            bytes.extend_from_slice(sender.as_bytes());
        }

        Message::new_binary(bytes)
    }
}

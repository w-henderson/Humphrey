use humphrey_ws::Message;

pub struct BroadcastMessage {
    pub message: String,
    pub sender: Option<String>,
}

impl BroadcastMessage {
    pub fn serialise(&self) -> Message {
        let mut bytes = Vec::with_capacity(
            self.message.len() + self.sender.as_ref().map(|s| s.len()).unwrap_or(0) + 8,
        );

        bytes.extend_from_slice(&self.message.len().to_be_bytes());
        bytes.extend_from_slice(self.message.as_bytes());

        if let Some(sender) = &self.sender {
            bytes.extend_from_slice(&sender.len().to_be_bytes());
            bytes.extend_from_slice(sender.as_bytes());
        } else {
            bytes.extend_from_slice(&[0; 4]);
        }

        Message::new_binary(bytes)
    }
}

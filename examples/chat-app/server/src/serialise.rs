use humphrey_ws::Message;

pub trait SerialisableMessage {
    fn serialise(&self) -> Message;
}

pub struct BroadcastMessage {
    pub message: String,
    pub sender_id: usize,
    pub sender_name: Option<String>,
}

pub struct ParticipantUpdateMessage {
    pub participants: Vec<String>,
}

impl SerialisableMessage for BroadcastMessage {
    fn serialise(&self) -> Message {
        let mut bytes = Vec::with_capacity(
            self.message.len() + self.sender_name.as_ref().map(|s| s.len()).unwrap_or(0) + 25,
        );

        bytes.push(0); // broadcast message type

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

impl SerialisableMessage for ParticipantUpdateMessage {
    fn serialise(&self) -> Message {
        let mut bytes = vec![1]; // participant update message type

        bytes.extend_from_slice(
            &self
                .participants
                .iter()
                .fold(0, |acc, s| acc + s.len() + 1)
                .to_be_bytes(),
        );

        for participant in &self.participants {
            bytes.extend_from_slice(participant.as_bytes());
            bytes.push(b'\n');
        }

        Message::new_binary(bytes)
    }
}

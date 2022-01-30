pub mod event;

use event::{Event, ToEventMask};

use std::sync::mpsc::Sender;

#[derive(Default)]
pub struct MonitorConfig {
    mask: u32,
    sender: Option<Sender<Event>>,
}

impl MonitorConfig {
    pub fn new(sender: Sender<Event>) -> Self {
        Self {
            mask: 0,
            sender: Some(sender),
        }
    }

    pub fn with_subscription_to<T>(mut self, event: T) -> Self
    where
        T: ToEventMask,
    {
        self.mask |= event.to_event_mask();
        self
    }

    pub fn send(&self, event: impl Into<Event>) {
        if let Some(sender) = &self.sender {
            let event = event.into();

            if self.mask & event.kind.to_event_mask() != 0 {
                sender.send(event).unwrap();
            }
        }
    }
}

impl Clone for MonitorConfig {
    fn clone(&self) -> Self {
        MonitorConfig {
            mask: self.mask,
            sender: self.sender.clone(),
        }
    }
}

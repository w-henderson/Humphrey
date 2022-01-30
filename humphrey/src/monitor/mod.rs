pub mod event;

use event::{Event, ToEventMask};

use std::sync::mpsc::Sender;

pub struct MonitorConfig {
    mask: u32,
    sender: Sender<Event>,
}

impl MonitorConfig {
    pub fn new(sender: Sender<Event>) -> Self {
        Self { mask: 0, sender }
    }

    pub fn with_subscription_to<T>(mut self, event: T) -> Self
    where
        T: ToEventMask,
    {
        self.mask |= event.to_event_mask();
        self
    }

    pub fn send(&self, event: Event) {
        if self.mask & event.kind.to_event_mask() != 0 {
            self.sender.send(event).unwrap();
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

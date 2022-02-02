//! Monitoring functionality.

pub mod event;

use event::{Event, ToEventMask};

use std::sync::mpsc::Sender;

/// Represents configuration for monitoring.
///
/// Cloning this type will create another instance which sends events to the same receiver,
///   like cloning a channel.
///
/// It is important to note that if the monitor is subscribed to the `EventType::ThreadPoolPanic`
///   event, panic messages will be relayed through the monitor instead of through the standard
///   error stream.
#[derive(Default)]
pub struct MonitorConfig {
    mask: u32,
    sender: Option<Sender<Event>>,
}

impl MonitorConfig {
    /// Creates a new monitor configuration with the given sender.
    pub fn new(sender: Sender<Event>) -> Self {
        Self {
            mask: 0,
            sender: Some(sender),
        }
    }

    /// Subscribes the monitor to the given event mask.
    /// This can be an event type or an event level.
    pub fn with_subscription_to<T>(mut self, event: T) -> Self
    where
        T: ToEventMask,
    {
        self.mask |= event.to_event_mask();
        self
    }

    /// Send a monitoring event.
    pub fn send(&self, event: impl Into<Event>) {
        if let Some(sender) = &self.sender {
            let event = event.into();

            if self.mask & event.kind.to_event_mask() != 0 {
                sender.send(event).ok();
            }
        }
    }

    /// Get the mask of the monitor.
    pub const fn mask(&self) -> u32 {
        self.mask
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

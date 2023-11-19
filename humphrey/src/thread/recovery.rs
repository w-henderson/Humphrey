//! Provides functionality for recovering from thread panics.

use crate::monitor::event::{Event, EventType};
use crate::monitor::MonitorConfig;
use crate::thread::pool::{Message, Thread};

use std::panic;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{panicking, spawn, JoinHandle};

/// Marker struct to detect thread panics.
pub struct PanicMarker(pub usize, pub Sender<Option<usize>>);

/// Manages the recovery thread.
pub struct RecoveryThread {
    /// handle
    pub handle: Option<JoinHandle<()>>,
    /// Sends a number of a thread or None to shutdown
    pub tx: Sender<Option<usize>>,
}

impl RecoveryThread {
    /// Creates and starts a new recovery thread.
    pub fn new(
        rx: Receiver<Option<usize>>,
        tx: Sender<Option<usize>>,
        task_rx: Arc<Mutex<Receiver<Message>>>,
        threads: Arc<Mutex<Vec<Thread>>>,
        monitor: Option<MonitorConfig>,
    ) -> Self {
        // If monitoring is enabled,
        if let Some(monitor) = monitor.clone() {
            // If the monitor is subscribed to the `ThreadPoolPanic` event,
            if monitor.mask() & EventType::ThreadPoolPanic as u32 != 0 {
                // The monitor needs to be `Sync` in order to be used in a panic handler.
                // Therefore, it needs to be behind a `Mutex`.
                // Therefore, we need a `&'static Mutex`, which we do by leaking a box.
                let sync_monitor = Mutex::new(monitor);
                let sync_monitor: &'static Mutex<MonitorConfig> = Box::leak(Box::new(sync_monitor));

                // Override the default panic handler to get more information about panics.
                panic::set_hook(Box::new(move |info| {
                    if let Ok(monitor) = sync_monitor.lock() {
                        if let Some(info) = info.location() {
                            monitor.send(Event::new(EventType::ThreadPoolPanic).with_info(
                                format!(
                                    "Thread {} panicked at {}:{}:{}",
                                    std::thread::current().name().unwrap_or("<unknown>"),
                                    info.file(),
                                    info.line(),
                                    info.column()
                                ),
                            ));
                        } else {
                            monitor.send(Event::new(EventType::ThreadPoolPanic).with_info(
                                format!(
                                    "Thread {} panicked, no location available",
                                    std::thread::current().name().unwrap_or("<unknown>"),
                                ),
                            ));
                        }
                    }
                }))
            };
        }

        let thread = spawn({
            let tx = tx.clone(); 
            move || {
            for panicking_thread in &rx {
                if let Some(panicking_thread) = panicking_thread {
                    let mut threads = threads.lock().unwrap();

                    // End the OS thread that panicked.
                    if let Some(thread) = threads[panicking_thread].os_thread.take() {
                        thread.join().ok();
                    }

                    // Start a new thread with the same ID.
                    let restarted_thread = Thread::new(
                        panicking_thread,
                        task_rx.clone(),
                        tx.clone(),
                        monitor.clone(),
                    );

                    // Put the new thread in the old thread's place.
                    threads[panicking_thread] = restarted_thread;

                    // Log that the thread restarted.
                    if let Some(monitor) = &monitor {
                        monitor.send(
                            Event::new(EventType::ThreadRestarted)
                                .with_info(format!("Thread {} was restarted", panicking_thread)),
                        );
                    }
                } else {
                    // stopping recovery
                    break;
                }
            }
        }
    });

        Self {
            handle: Some(thread), tx
        }
    }
}

impl Drop for PanicMarker {
    fn drop(&mut self) {
        if panicking() {
            self.1.send(Some(self.0)).ok();
        }
    }
}

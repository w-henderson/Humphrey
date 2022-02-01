//! Provides thread pool functionality.

use crate::monitor::event::EventType;
use crate::monitor::MonitorConfig;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{spawn, JoinHandle};
use std::time::Instant;

/// The number of milliseconds a task can be waiting in the pool before the pool is considered overloaded.
const OVERLOAD_THRESHOLD: u128 = 100;

/// Represents a pool of threads.
pub struct ThreadPool {
    thread_count: usize,
    threads: Vec<Thread>,
    tx: Sender<Message>,
    monitor: MonitorConfig,
}

/// Represents a single worker thread in the thread pool
pub struct Thread {
    #[allow(dead_code)]
    id: usize,
    os_thread: Option<JoinHandle<()>>,
}

/// Represents a message between threads.
pub enum Message {
    /// A task to be processed by a thread.
    Function(Task, Instant),
    /// The thread should be shut down.
    Shutdown,
}

trait CallableTask {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> CallableTask for F {
    fn call_box(self: Box<Self>) {
        (*self)();
    }
}

/// Represents a task to run in the thread pool.
pub type Task = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Creates a new thread pool with the given number of threads.
    ///
    /// ## Panics
    /// This function will panic if the number of threads is zero.
    pub fn new(thread_count: usize) -> Self {
        assert!(thread_count > 0);

        Self {
            thread_count,
            threads: Vec::new(),
            tx: channel().0,
            monitor: MonitorConfig::new(channel().0),
        }
    }

    /// Starts the thread pool.
    pub fn start(&mut self) {
        let (tx, rx): (Sender<Message>, Receiver<Message>) = channel();
        let rx = Arc::new(Mutex::new(rx));
        self.threads = Vec::with_capacity(self.thread_count);

        for id in 0..self.thread_count {
            self.threads
                .push(Thread::new(id, rx.clone(), self.monitor.clone()))
        }

        self.tx = tx;
    }

    pub fn register_monitor(&mut self, monitor: MonitorConfig) {
        self.monitor = monitor;
    }

    /// Executes a task in the thread pool.
    ///
    /// ## Panics
    /// This function will panic if the thread pool has not been started.
    pub fn execute<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        assert!(!self.threads.is_empty());

        let boxed_task = Box::new(task);
        let time_into_pool = Instant::now();
        self.tx
            .send(Message::Function(boxed_task, time_into_pool))
            .unwrap();
    }
}

impl Thread {
    /// Creates a new thread.
    pub fn new(id: usize, rx: Arc<Mutex<Receiver<Message>>>, monitor: MonitorConfig) -> Self {
        let thread = spawn(move || loop {
            let task = { rx.lock().unwrap().recv().unwrap() };

            match task {
                Message::Function(f, t) => {
                    let time_in_pool = t.elapsed().as_millis();

                    if time_in_pool > OVERLOAD_THRESHOLD {
                        monitor.send(EventType::ThreadPoolOverload);
                    }

                    (f)()
                }
                Message::Shutdown => break,
            }
        });

        Self {
            id,
            os_thread: Some(thread),
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for thread in &mut self.threads {
            if let Some(thread) = thread.os_thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

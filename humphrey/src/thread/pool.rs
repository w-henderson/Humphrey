use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{spawn, JoinHandle};

/// Represents a pool of threads.
pub struct ThreadPool {
    threads: Vec<Thread>,
    tx: Sender<Message>,
}

/// Represents a single worker thread in the thread pool
pub struct Thread {
    #[allow(dead_code)]
    id: usize,
    os_thread: Option<JoinHandle<()>>,
}

/// Represents a message between threads.
pub enum Message {
    Function(Task),
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
    /// The thread count must be above zero.
    pub fn new(thread_count: usize) -> Self {
        assert!(thread_count > 0);

        let (tx, rx): (Sender<Message>, Receiver<Message>) = channel();
        let rx = Arc::new(Mutex::new(rx));
        let mut threads = Vec::with_capacity(thread_count);

        for id in 0..thread_count {
            threads.push(Thread::new(id, rx.clone()))
        }

        ThreadPool { threads, tx }
    }

    /// Executes a task in the thread pool.
    pub fn execute<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let boxed_task = Box::new(task);
        self.tx.send(Message::Function(boxed_task)).unwrap();
    }
}

impl Thread {
    /// Creates a new thread.
    pub fn new(id: usize, rx: Arc<Mutex<Receiver<Message>>>) -> Self {
        let thread = spawn(move || loop {
            let task = { rx.lock().unwrap().recv().unwrap() };

            match task {
                Message::Function(f) => (f)(),
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

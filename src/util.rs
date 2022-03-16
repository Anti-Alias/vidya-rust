use std::collections::VecDeque;

/// Represents a queue of signals
#[derive(Debug)]
pub struct SignalQueue<S: Clone + Into<u32>> {
    queue: VecDeque<S>
}
impl<S: Clone + Into<u32>> SignalQueue<S> {

    pub fn new() -> Self {
        Self { queue: VecDeque::new() }
    }

    // Pushes signal into the queue.
    // Returns false if there was a duplicate
    pub fn push(&mut self, signal: S) -> bool {
        let signal_id: u32 = signal.clone().into();
        let any_duplicates =  self.queue.iter().any(|sig| {
            let sig_id: u32 = sig.clone().into();
            signal_id == sig_id
        });
        if any_duplicates {
            return false
        }
        self.queue.push_back(signal);
        true
    }

    /// Removes a signal from the queue.
    pub fn pop(&mut self) -> Option<S> {
        self.queue.pop_front()
    }

    pub fn len(&self) -> usize { self.queue.len() }

    pub fn is_empty(&self) -> bool { self.queue.is_empty() }
}
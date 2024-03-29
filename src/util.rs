use std::collections::VecDeque;

/// Represents a queue of signals
#[derive(Debug)]
pub struct SignalQueue<S: Clone> {
    queue: VecDeque<S>
}
impl<S: Clone> SignalQueue<S> {

    pub fn new() -> Self {
        Self { queue: VecDeque::new() }
    }

    // Pushes signal into the queue.
    // Returns false if there was a duplicate
    pub fn push(&mut self, signal: S) -> bool {
        let signal_disc = std::mem::discriminant(&signal);
        let duplicate_signal = self.queue.iter().any(|sig| {
            let sig_disc = std::mem::discriminant(sig);
            signal_disc == sig_disc
        });
        if !duplicate_signal {
            self.queue.push_back(signal);
        }
        true
    }

    /// Removes a signal from the queue.
    pub fn pop(&mut self) -> Option<S> {
        self.queue.pop_front()
    }

    pub fn len(&self) -> usize { self.queue.len() }

    pub fn is_empty(&self) -> bool { self.queue.is_empty() }
}
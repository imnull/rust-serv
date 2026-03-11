//! Counter metric - monotonically increasing value

use std::sync::atomic::{AtomicU64, Ordering};

/// A counter is a cumulative metric that represents a single monotonically increasing counter
/// whose value can only increase or be reset to zero.
#[derive(Debug)]
pub struct Counter {
    value: AtomicU64,
    name: String,
    help: String,
}

impl Counter {
    /// Create a new counter with name and help text
    pub fn new(name: impl Into<String>, help: impl Into<String>) -> Self {
        Self {
            value: AtomicU64::new(0),
            name: name.into(),
            help: help.into(),
        }
    }

    /// Increment the counter by 1
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment the counter by a specific amount
    pub fn add(&self, delta: u64) {
        self.value.fetch_add(delta, Ordering::Relaxed);
    }

    /// Get the current value
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    /// Reset the counter to zero
    pub fn reset(&self) {
        self.value.store(0, Ordering::Relaxed);
    }

    /// Get the counter name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the help text
    pub fn help(&self) -> &str {
        &self.help
    }
}

impl Clone for Counter {
    fn clone(&self) -> Self {
        Self {
            value: AtomicU64::new(self.value.load(Ordering::Relaxed)),
            name: self.name.clone(),
            help: self.help.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_creation() {
        let counter = Counter::new("requests_total", "Total number of requests");
        assert_eq!(counter.get(), 0);
        assert_eq!(counter.name(), "requests_total");
        assert_eq!(counter.help(), "Total number of requests");
    }

    #[test]
    fn test_counter_inc() {
        let counter = Counter::new("test", "test counter");
        counter.inc();
        assert_eq!(counter.get(), 1);
        
        counter.inc();
        counter.inc();
        assert_eq!(counter.get(), 3);
    }

    #[test]
    fn test_counter_add() {
        let counter = Counter::new("test", "test counter");
        counter.add(10);
        assert_eq!(counter.get(), 10);
        
        counter.add(5);
        assert_eq!(counter.get(), 15);
    }

    #[test]
    fn test_counter_reset() {
        let counter = Counter::new("test", "test counter");
        counter.inc();
        counter.inc();
        assert_eq!(counter.get(), 2);
        
        counter.reset();
        assert_eq!(counter.get(), 0);
    }

    #[test]
    fn test_counter_clone() {
        let counter = Counter::new("test", "test counter");
        counter.inc();
        
        let cloned = counter.clone();
        assert_eq!(cloned.get(), 1);
        assert_eq!(cloned.name(), "test");
    }

    #[test]
    fn test_counter_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let counter = Arc::new(Counter::new("test", "test counter"));
        let mut handles = vec![];

        for _ in 0..10 {
            let counter_clone = Arc::clone(&counter);
            handles.push(thread::spawn(move || {
                counter_clone.inc();
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(counter.get(), 10);
    }

    #[test]
    fn test_counter_large_values() {
        let counter = Counter::new("test", "test counter");
        counter.add(1_000_000);
        assert_eq!(counter.get(), 1_000_000);
    }
}

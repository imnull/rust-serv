//! Gauge metric - value that can go up or down

use std::sync::atomic::{AtomicI64, Ordering};

/// A gauge is a metric that represents a single numerical value that can arbitrarily go up and down.
#[derive(Debug)]
pub struct Gauge {
    value: AtomicI64,
    name: String,
    help: String,
}

impl Gauge {
    /// Create a new gauge with name and help text
    pub fn new(name: impl Into<String>, help: impl Into<String>) -> Self {
        Self {
            value: AtomicI64::new(0),
            name: name.into(),
            help: help.into(),
        }
    }

    /// Set the gauge to a specific value
    pub fn set(&self, value: i64) {
        self.value.store(value, Ordering::Relaxed);
    }

    /// Increment the gauge by 1
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement the gauge by 1
    pub fn dec(&self) {
        self.value.fetch_sub(1, Ordering::Relaxed);
    }

    /// Add a value to the gauge
    pub fn add(&self, delta: i64) {
        self.value.fetch_add(delta, Ordering::Relaxed);
    }

    /// Subtract a value from the gauge
    pub fn sub(&self, delta: i64) {
        self.value.fetch_sub(delta, Ordering::Relaxed);
    }

    /// Get the current value
    pub fn get(&self) -> i64 {
        self.value.load(Ordering::Relaxed)
    }

    /// Get the gauge name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the help text
    pub fn help(&self) -> &str {
        &self.help
    }
}

impl Clone for Gauge {
    fn clone(&self) -> Self {
        Self {
            value: AtomicI64::new(self.value.load(Ordering::Relaxed)),
            name: self.name.clone(),
            help: self.help.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gauge_creation() {
        let gauge = Gauge::new("active_connections", "Number of active connections");
        assert_eq!(gauge.get(), 0);
        assert_eq!(gauge.name(), "active_connections");
        assert_eq!(gauge.help(), "Number of active connections");
    }

    #[test]
    fn test_gauge_set() {
        let gauge = Gauge::new("test", "test gauge");
        gauge.set(42);
        assert_eq!(gauge.get(), 42);
        
        gauge.set(100);
        assert_eq!(gauge.get(), 100);
    }

    #[test]
    fn test_gauge_inc_dec() {
        let gauge = Gauge::new("test", "test gauge");
        gauge.inc();
        assert_eq!(gauge.get(), 1);
        
        gauge.inc();
        assert_eq!(gauge.get(), 2);
        
        gauge.dec();
        assert_eq!(gauge.get(), 1);
    }

    #[test]
    fn test_gauge_add_sub() {
        let gauge = Gauge::new("test", "test gauge");
        gauge.add(10);
        assert_eq!(gauge.get(), 10);
        
        gauge.sub(3);
        assert_eq!(gauge.get(), 7);
    }

    #[test]
    fn test_gauge_negative_values() {
        let gauge = Gauge::new("test", "test gauge");
        gauge.set(-5);
        assert_eq!(gauge.get(), -5);
        
        gauge.add(10);
        assert_eq!(gauge.get(), 5);
        
        gauge.sub(20);
        assert_eq!(gauge.get(), -15);
    }

    #[test]
    fn test_gauge_clone() {
        let gauge = Gauge::new("test", "test gauge");
        gauge.set(50);
        
        let cloned = gauge.clone();
        assert_eq!(cloned.get(), 50);
        assert_eq!(cloned.name(), "test");
    }

    #[test]
    fn test_gauge_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let gauge = Arc::new(Gauge::new("test", "test gauge"));
        let mut handles = vec![];

        for _ in 0..5 {
            let gauge_clone = Arc::clone(&gauge);
            handles.push(thread::spawn(move || {
                gauge_clone.inc();
            }));
        }

        for _ in 0..3 {
            let gauge_clone = Arc::clone(&gauge);
            handles.push(thread::spawn(move || {
                gauge_clone.dec();
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // 5 inc - 3 dec = 2
        assert_eq!(gauge.get(), 2);
    }
}

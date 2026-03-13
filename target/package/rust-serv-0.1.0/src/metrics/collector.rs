//! Central metrics collector

use std::sync::Arc;
use std::collections::HashMap;

use super::counter::Counter;
use super::gauge::Gauge;
use super::histogram::Histogram;

/// Central metrics collector that manages all metrics
pub struct MetricsCollector {
    counters: RwLock<HashMap<String, Arc<Counter>>>,
    gauges: RwLock<HashMap<String, Arc<Gauge>>>,
    histograms: RwLock<HashMap<String, Arc<Histogram>>>,
}

use std::sync::RwLock;

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            counters: RwLock::new(HashMap::new()),
            gauges: RwLock::new(HashMap::new()),
            histograms: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new counter
    pub fn create_counter(&self, name: impl Into<String>, help: impl Into<String>) -> Arc<Counter> {
        let counter = Arc::new(Counter::new(name.into(), help.into()));
        let name = counter.name().to_string();
        
        let mut counters = self.counters.write().unwrap();
        counters.insert(name, Arc::clone(&counter));
        
        counter
    }

    /// Create a new gauge
    pub fn create_gauge(&self, name: impl Into<String>, help: impl Into<String>) -> Arc<Gauge> {
        let gauge = Arc::new(Gauge::new(name.into(), help.into()));
        let name = gauge.name().to_string();
        
        let mut gauges = self.gauges.write().unwrap();
        gauges.insert(name, Arc::clone(&gauge));
        
        gauge
    }

    /// Create a new histogram
    pub fn create_histogram(&self, name: impl Into<String>, help: impl Into<String>) -> Arc<Histogram> {
        let histogram = Arc::new(Histogram::new(name.into(), help.into()));
        let name = histogram.name().to_string();
        
        let mut histograms = self.histograms.write().unwrap();
        histograms.insert(name, Arc::clone(&histogram));
        
        histogram
    }

    /// Get a counter by name
    pub fn get_counter(&self, name: &str) -> Option<Arc<Counter>> {
        let counters = self.counters.read().unwrap();
        counters.get(name).cloned()
    }

    /// Get a gauge by name
    pub fn get_gauge(&self, name: &str) -> Option<Arc<Gauge>> {
        let gauges = self.gauges.read().unwrap();
        gauges.get(name).cloned()
    }

    /// Get a histogram by name
    pub fn get_histogram(&self, name: &str) -> Option<Arc<Histogram>> {
        let histograms = self.histograms.read().unwrap();
        histograms.get(name).cloned()
    }

    /// Get all counters
    pub fn all_counters(&self) -> Vec<Arc<Counter>> {
        let counters = self.counters.read().unwrap();
        counters.values().cloned().collect()
    }

    /// Get all gauges
    pub fn all_gauges(&self) -> Vec<Arc<Gauge>> {
        let gauges = self.gauges.read().unwrap();
        gauges.values().cloned().collect()
    }

    /// Get all histograms
    pub fn all_histograms(&self) -> Vec<Arc<Histogram>> {
        let histograms = self.histograms.read().unwrap();
        histograms.values().cloned().collect()
    }

    /// Remove a counter by name
    pub fn remove_counter(&self, name: &str) -> bool {
        let mut counters = self.counters.write().unwrap();
        counters.remove(name).is_some()
    }

    /// Remove a gauge by name
    pub fn remove_gauge(&self, name: &str) -> bool {
        let mut gauges = self.gauges.write().unwrap();
        gauges.remove(name).is_some()
    }

    /// Remove a histogram by name
    pub fn remove_histogram(&self, name: &str) -> bool {
        let mut histograms = self.histograms.write().unwrap();
        histograms.remove(name).is_some()
    }

    /// Clear all metrics
    pub fn clear(&self) {
        let mut counters = self.counters.write().unwrap();
        let mut gauges = self.gauges.write().unwrap();
        let mut histograms = self.histograms.write().unwrap();
        
        counters.clear();
        gauges.clear();
        histograms.clear();
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collector_creation() {
        let collector = MetricsCollector::new();
        assert!(collector.all_counters().is_empty());
        assert!(collector.all_gauges().is_empty());
        assert!(collector.all_histograms().is_empty());
    }

    #[test]
    fn test_create_counter() {
        let collector = MetricsCollector::new();
        let counter = collector.create_counter("requests_total", "Total requests");
        
        counter.inc();
        assert_eq!(counter.get(), 1);
        assert!(collector.get_counter("requests_total").is_some());
    }

    #[test]
    fn test_create_gauge() {
        let collector = MetricsCollector::new();
        let gauge = collector.create_gauge("active_connections", "Active connections");
        
        gauge.set(10);
        assert_eq!(gauge.get(), 10);
        assert!(collector.get_gauge("active_connections").is_some());
    }

    #[test]
    fn test_create_histogram() {
        let collector = MetricsCollector::new();
        let hist = collector.create_histogram("request_duration", "Request duration");
        
        hist.observe(0.1);
        assert_eq!(hist.count(), 1);
        assert!(collector.get_histogram("request_duration").is_some());
    }

    #[test]
    fn test_get_nonexistent() {
        let collector = MetricsCollector::new();
        assert!(collector.get_counter("nonexistent").is_none());
        assert!(collector.get_gauge("nonexistent").is_none());
        assert!(collector.get_histogram("nonexistent").is_none());
    }

    #[test]
    fn test_all_counters() {
        let collector = MetricsCollector::new();
        collector.create_counter("counter1", "Counter 1");
        collector.create_counter("counter2", "Counter 2");
        
        let counters = collector.all_counters();
        assert_eq!(counters.len(), 2);
    }

    #[test]
    fn test_all_gauges() {
        let collector = MetricsCollector::new();
        collector.create_gauge("gauge1", "Gauge 1");
        collector.create_gauge("gauge2", "Gauge 2");
        collector.create_gauge("gauge3", "Gauge 3");
        
        let gauges = collector.all_gauges();
        assert_eq!(gauges.len(), 3);
    }

    #[test]
    fn test_all_histograms() {
        let collector = MetricsCollector::new();
        collector.create_histogram("hist1", "Histogram 1");
        
        let histograms = collector.all_histograms();
        assert_eq!(histograms.len(), 1);
    }

    #[test]
    fn test_remove_counter() {
        let collector = MetricsCollector::new();
        collector.create_counter("test_counter", "Test");
        
        assert!(collector.get_counter("test_counter").is_some());
        assert!(collector.remove_counter("test_counter"));
        assert!(collector.get_counter("test_counter").is_none());
    }

    #[test]
    fn test_remove_nonexistent() {
        let collector = MetricsCollector::new();
        assert!(!collector.remove_counter("nonexistent"));
    }

    #[test]
    fn test_clear() {
        let collector = MetricsCollector::new();
        collector.create_counter("counter1", "Counter 1");
        collector.create_gauge("gauge1", "Gauge 1");
        collector.create_histogram("hist1", "Histogram 1");
        
        collector.clear();
        
        assert!(collector.all_counters().is_empty());
        assert!(collector.all_gauges().is_empty());
        assert!(collector.all_histograms().is_empty());
    }

    #[test]
    fn test_shared_counter() {
        let collector = MetricsCollector::new();
        let counter = collector.create_counter("test", "Test");
        
        let counter2 = collector.get_counter("test").unwrap();
        
        counter.inc();
        counter2.inc();
        
        assert_eq!(counter.get(), 2);
        assert_eq!(counter2.get(), 2);
    }

    #[test]
    fn test_default() {
        let collector = MetricsCollector::default();
        assert!(collector.all_counters().is_empty());
    }
}

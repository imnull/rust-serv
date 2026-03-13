//! Prometheus format exporter

use super::collector::MetricsCollector;
use super::counter::Counter;
use super::gauge::Gauge;
use super::histogram::Histogram;

/// Prometheus text format exporter
pub struct PrometheusExporter {
    collector: MetricsCollector,
    namespace: String,
}

impl PrometheusExporter {
    /// Create a new Prometheus exporter
    pub fn new(collector: MetricsCollector) -> Self {
        Self {
            collector,
            namespace: "rust_serv".to_string(),
        }
    }

    /// Create a new Prometheus exporter with custom namespace
    pub fn with_namespace(collector: MetricsCollector, namespace: impl Into<String>) -> Self {
        Self {
            collector,
            namespace: namespace.into(),
        }
    }

    /// Export metrics in Prometheus text format
    pub fn export(&self) -> String {
        let mut output = String::new();
        
        // Export counters
        for counter in self.collector.all_counters() {
            output.push_str(&self.format_counter(&counter));
            output.push('\n');
        }
        
        // Export gauges
        for gauge in self.collector.all_gauges() {
            output.push_str(&self.format_gauge(&gauge));
            output.push('\n');
        }
        
        // Export histograms
        for histogram in self.collector.all_histograms() {
            output.push_str(&self.format_histogram(&histogram));
            output.push('\n');
        }
        
        output.trim_end().to_string()
    }

    /// Format a counter in Prometheus format
    fn format_counter(&self, counter: &Counter) -> String {
        let metric_name = self.namespaced_name(counter.name());
        format!(
            "# HELP {} {}\n# TYPE {} counter\n{} {}",
            metric_name,
            counter.help(),
            metric_name,
            metric_name,
            counter.get()
        )
    }

    /// Format a gauge in Prometheus format
    fn format_gauge(&self, gauge: &Gauge) -> String {
        let metric_name = self.namespaced_name(gauge.name());
        format!(
            "# HELP {} {}\n# TYPE {} gauge\n{} {}",
            metric_name,
            gauge.help(),
            metric_name,
            metric_name,
            gauge.get()
        )
    }

    /// Format a histogram in Prometheus format
    fn format_histogram(&self, histogram: &Histogram) -> String {
        let base_name = self.namespaced_name(histogram.name());
        let mut output = String::new();
        
        // HELP and TYPE
        output.push_str(&format!("# HELP {} {}\n", base_name, histogram.help()));
        output.push_str(&format!("# TYPE {} histogram\n", base_name));
        
        // Bucket lines (cumulative)
        let bucket_counts = histogram.bucket_counts();
        let boundaries = histogram.boundaries();
        
        for (idx, count) in bucket_counts.iter().enumerate() {
            if idx < boundaries.len() {
                output.push_str(&format!(
                    "{}_bucket{{le=\"{}\"}} {}\n",
                    base_name, boundaries[idx], count
                ));
            } else {
                // +Inf bucket
                output.push_str(&format!(
                    "{}_bucket{{le=\"+Inf\"}} {}\n",
                    base_name, count
                ));
            }
        }
        
        // Sum and count
        output.push_str(&format!("{}_sum {}\n", base_name, histogram.sum()));
        output.push_str(&format!("{}_count {}", base_name, histogram.count()));
        
        output.trim_end().to_string()
    }

    /// Create a namespaced metric name
    fn namespaced_name(&self, name: &str) -> String {
        format!("{}_{}", self.namespace, name)
    }

    /// Get the collector
    pub fn collector(&self) -> &MetricsCollector {
        &self.collector
    }

    /// Get mutable collector
    pub fn collector_mut(&mut self) -> &mut MetricsCollector {
        &mut self.collector
    }

    /// Get the namespace
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Set the namespace
    pub fn set_namespace(&mut self, namespace: impl Into<String>) {
        self.namespace = namespace.into();
    }
}

impl Clone for PrometheusExporter {
    fn clone(&self) -> Self {
        Self {
            collector: MetricsCollector::new(),
            namespace: self.namespace.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_collector() -> MetricsCollector {
        MetricsCollector::new()
    }

    #[test]
    fn test_exporter_creation() {
        let collector = create_test_collector();
        let exporter = PrometheusExporter::new(collector);
        assert_eq!(exporter.namespace(), "rust_serv");
    }

    #[test]
    fn test_exporter_with_namespace() {
        let collector = create_test_collector();
        let exporter = PrometheusExporter::with_namespace(collector, "my_app");
        assert_eq!(exporter.namespace(), "my_app");
    }

    #[test]
    fn test_export_empty() {
        let collector = create_test_collector();
        let exporter = PrometheusExporter::new(collector);
        let output = exporter.export();
        assert!(output.is_empty());
    }

    #[test]
    fn test_export_counter() {
        let collector = create_test_collector();
        let counter = collector.create_counter("requests_total", "Total requests");
        counter.inc();
        counter.inc();
        
        let exporter = PrometheusExporter::new(collector);
        let output = exporter.export();
        
        assert!(output.contains("# HELP rust_serv_requests_total Total requests"));
        assert!(output.contains("# TYPE rust_serv_requests_total counter"));
        assert!(output.contains("rust_serv_requests_total 2"));
    }

    #[test]
    fn test_export_gauge() {
        let collector = create_test_collector();
        let gauge = collector.create_gauge("active_connections", "Active connections");
        gauge.set(42);
        
        let exporter = PrometheusExporter::new(collector);
        let output = exporter.export();
        
        assert!(output.contains("# HELP rust_serv_active_connections Active connections"));
        assert!(output.contains("# TYPE rust_serv_active_connections gauge"));
        assert!(output.contains("rust_serv_active_connections 42"));
    }

    #[test]
    fn test_export_histogram() {
        let collector = create_test_collector();
        let hist = collector.create_histogram("request_duration", "Request duration");
        hist.observe(0.05);
        hist.observe(0.15);
        
        let exporter = PrometheusExporter::new(collector);
        let output = exporter.export();
        
        assert!(output.contains("# HELP rust_serv_request_duration Request duration"));
        assert!(output.contains("# TYPE rust_serv_request_duration histogram"));
        assert!(output.contains("rust_serv_request_duration_bucket"));
        assert!(output.contains("rust_serv_request_duration_sum"));
        assert!(output.contains("rust_serv_request_duration_count"));
        assert!(output.contains("le=\"+Inf\""));
    }

    #[test]
    fn test_export_multiple_metrics() {
        let collector = create_test_collector();
        
        let counter = collector.create_counter("requests", "Total requests");
        counter.inc();
        
        let gauge = collector.create_gauge("connections", "Active connections");
        gauge.set(5);
        
        let exporter = PrometheusExporter::new(collector);
        let output = exporter.export();
        
        assert!(output.contains("rust_serv_requests 1"));
        assert!(output.contains("rust_serv_connections 5"));
    }

    #[test]
    fn test_set_namespace() {
        let collector = create_test_collector();
        let mut exporter = PrometheusExporter::new(collector);
        exporter.set_namespace("new_namespace");
        
        assert_eq!(exporter.namespace(), "new_namespace");
    }

    #[test]
    fn test_exporter_clone() {
        let collector = create_test_collector();
        let exporter = PrometheusExporter::with_namespace(collector, "test_ns");
        
        let cloned = exporter.clone();
        assert_eq!(cloned.namespace(), "test_ns");
    }

    #[test]
    fn test_collector_access() {
        let collector = create_test_collector();
        let mut exporter = PrometheusExporter::new(collector);
        
        // Access collector
        let _ = exporter.collector();
        
        // Access mutable collector
        let collector = exporter.collector_mut();
        collector.create_counter("new_counter", "New counter");
        
        assert!(exporter.collector().get_counter("new_counter").is_some());
    }

    #[test]
    fn test_histogram_bucket_cumulative() {
        let collector = create_test_collector();
        let hist = collector.create_histogram("latency", "Latency");
        
        // Observe values in different buckets
        hist.observe(0.001);
        hist.observe(0.05);
        hist.observe(0.5);
        
        let exporter = PrometheusExporter::new(collector);
        let output = exporter.export();
        
        assert!(output.contains("rust_serv_latency_bucket"));
        assert!(output.contains("rust_serv_latency_count 3"));
    }

    #[test]
    fn test_negative_gauge_value() {
        let collector = create_test_collector();
        let gauge = collector.create_gauge("temperature", "Temperature");
        gauge.set(-10);
        
        let exporter = PrometheusExporter::new(collector);
        let output = exporter.export();
        
        assert!(output.contains("rust_serv_temperature -10"));
    }

    #[test]
    fn test_large_counter_value() {
        let collector = create_test_collector();
        let counter = collector.create_counter("bytes", "Total bytes");
        counter.add(1_000_000_000);
        
        let exporter = PrometheusExporter::new(collector);
        let output = exporter.export();
        
        assert!(output.contains("rust_serv_bytes 1000000000"));
    }
}

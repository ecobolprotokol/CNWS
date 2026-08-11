//! Metrics collection for CNWS observability
//! Implements Prometheus metrics matching spec §20

use crate::error::Result;
use prometheus::{
    Counter, CounterVec, Encoder, Gauge, GaugeVec, Histogram, HistogramOpts,
    Opts, Registry, TextEncoder,
};

/// CNWS metrics collector
pub struct CnwsMetrics {
    registry: Registry,

    // Store metrics
    store_tiles_total: Gauge,
    store_segments_total: Gauge,
    store_size_bytes: Gauge,
    store_compressed_bytes: Gauge,

    // Operation metrics
    tile_reads_total: Counter,
    tile_writes_total: Counter,
    tile_deletes_total: Counter,
    operation_errors_total: CounterVec,

    // Latency metrics
    tile_read_latency: Histogram,
    tile_write_latency: Histogram,
    query_latency: Histogram,

    // Cache metrics
    cache_hits_total: Counter,
    cache_misses_total: Counter,
    cache_size_bytes: GaugeVec,

    // Memory metrics
    memory_entries_total: GaugeVec,
    memory_reads_total: CounterVec,
    memory_writes_total: CounterVec,

    // Revision metrics
    revision_commits_total: Counter,
    revision_checkouts_total: Counter,

    // GC metrics
    gc_runs_total: Counter,
    gc_tiles_freed: Counter,
    gc_duration_seconds: Histogram,

    // Recovery metrics
    recovery_runs_total: Counter,
    recovery_success_total: Counter,
    recovery_failure_total: Counter,
    recovery_duration_seconds: Histogram,

    // System metrics
    active_operations: Gauge,
    compute_budget_remaining: Gauge,
}

impl CnwsMetrics {
    /// Create a new metrics collector
    pub fn new() -> Result<Self> {
        let registry = Registry::new();

        // Store metrics
        let store_tiles_total = Gauge::with_opts(
            Opts::new("cnws_store_tiles_total", "Total number of tiles in store")
        )?;
        let store_segments_total = Gauge::with_opts(
            Opts::new("cnws_store_segments_total", "Total number of segments in store")
        )?;
        let store_size_bytes = Gauge::with_opts(
            Opts::new("cnws_store_size_bytes", "Total store size in bytes")
        )?;
        let store_compressed_bytes = Gauge::with_opts(
            Opts::new("cnws_store_compressed_bytes", "Total compressed size in bytes")
        )?;

        registry.register(Box::new(store_tiles_total.clone()))?;
        registry.register(Box::new(store_segments_total.clone()))?;
        registry.register(Box::new(store_size_bytes.clone()))?;
        registry.register(Box::new(store_compressed_bytes.clone()))?;

        // Operation metrics
        let tile_reads_total = Counter::with_opts(
            Opts::new("cnws_tile_reads_total", "Total number of tile reads")
        )?;
        let tile_writes_total = Counter::with_opts(
            Opts::new("cnws_tile_writes_total", "Total number of tile writes")
        )?;
        let tile_deletes_total = Counter::with_opts(
            Opts::new("cnws_tile_deletes_total", "Total number of tile deletes")
        )?;
        let operation_errors_total = CounterVec::new(
            Opts::new("cnws_operation_errors_total", "Total number of operation errors by type"),
            &["error_type"]
        )?;

        registry.register(Box::new(tile_reads_total.clone()))?;
        registry.register(Box::new(tile_writes_total.clone()))?;
        registry.register(Box::new(tile_deletes_total.clone()))?;
        registry.register(Box::new(operation_errors_total.clone()))?;

        // Latency metrics
        let tile_read_latency = Histogram::with_opts(
            HistogramOpts::new("cnws_tile_read_latency_seconds", "Tile read latency in seconds")
        )?;
        let tile_write_latency = Histogram::with_opts(
            HistogramOpts::new("cnws_tile_write_latency_seconds", "Tile write latency in seconds")
        )?;
        let query_latency = Histogram::with_opts(
            HistogramOpts::new("cnws_query_latency_seconds", "Query latency in seconds")
        )?;

        registry.register(Box::new(tile_read_latency.clone()))?;
        registry.register(Box::new(tile_write_latency.clone()))?;
        registry.register(Box::new(query_latency.clone()))?;

        // Cache metrics
        let cache_hits_total = Counter::with_opts(
            Opts::new("cnws_cache_hits_total", "Total number of cache hits")
        )?;
        let cache_misses_total = Counter::with_opts(
            Opts::new("cnws_cache_misses_total", "Total number of cache misses")
        )?;
        let cache_size_bytes = GaugeVec::new(
            Opts::new("cnws_cache_size_bytes", "Cache size in bytes by level"),
            &["level"]
        )?;

        registry.register(Box::new(cache_hits_total.clone()))?;
        registry.register(Box::new(cache_misses_total.clone()))?;
        registry.register(Box::new(cache_size_bytes.clone()))?;

        // Memory metrics
        let memory_entries_total = GaugeVec::new(
            Opts::new("cnws_memory_entries_total", "Total memory entries by type"),
            &["memory_type"]
        )?;
        let memory_reads_total = CounterVec::new(
            Opts::new("cnws_memory_reads_total", "Total memory reads by type"),
            &["memory_type"]
        )?;
        let memory_writes_total = CounterVec::new(
            Opts::new("cnws_memory_writes_total", "Total memory writes by type"),
            &["memory_type"]
        )?;

        registry.register(Box::new(memory_entries_total.clone()))?;
        registry.register(Box::new(memory_reads_total.clone()))?;
        registry.register(Box::new(memory_writes_total.clone()))?;

        // Revision metrics
        let revision_commits_total = Counter::with_opts(
            Opts::new("cnws_revision_commits_total", "Total number of revision commits")
        )?;
        let revision_checkouts_total = Counter::with_opts(
            Opts::new("cnws_revision_checkouts_total", "Total number of revision checkouts")
        )?;

        registry.register(Box::new(revision_commits_total.clone()))?;
        registry.register(Box::new(revision_checkouts_total.clone()))?;

        // GC metrics
        let gc_runs_total = Counter::with_opts(
            Opts::new("cnws_gc_runs_total", "Total number of GC runs")
        )?;
        let gc_tiles_freed = Counter::with_opts(
            Opts::new("cnws_gc_tiles_freed", "Total number of tiles freed by GC")
        )?;
        let gc_duration_seconds = Histogram::with_opts(
            HistogramOpts::new("cnws_gc_duration_seconds", "GC duration in seconds")
        )?;

        registry.register(Box::new(gc_runs_total.clone()))?;
        registry.register(Box::new(gc_tiles_freed.clone()))?;
        registry.register(Box::new(gc_duration_seconds.clone()))?;

        // Recovery metrics
        let recovery_runs_total = Counter::with_opts(
            Opts::new("cnws_recovery_runs_total", "Total number of recovery runs")
        )?;
        let recovery_success_total = Counter::with_opts(
            Opts::new("cnws_recovery_success_total", "Total number of successful recoveries")
        )?;
        let recovery_failure_total = Counter::with_opts(
            Opts::new("cnws_recovery_failure_total", "Total number of failed recoveries")
        )?;
        let recovery_duration_seconds = Histogram::with_opts(
            HistogramOpts::new("cnws_recovery_duration_seconds", "Recovery duration in seconds")
        )?;

        registry.register(Box::new(recovery_runs_total.clone()))?;
        registry.register(Box::new(recovery_success_total.clone()))?;
        registry.register(Box::new(recovery_failure_total.clone()))?;
        registry.register(Box::new(recovery_duration_seconds.clone()))?;

        // System metrics
        let active_operations = Gauge::with_opts(
            Opts::new("cnws_active_operations", "Number of active operations")
        )?;
        let compute_budget_remaining = Gauge::with_opts(
            Opts::new("cnws_compute_budget_remaining", "Remaining compute budget")
        )?;

        registry.register(Box::new(active_operations.clone()))?;
        registry.register(Box::new(compute_budget_remaining.clone()))?;

        Ok(Self {
            registry,
            store_tiles_total,
            store_segments_total,
            store_size_bytes,
            store_compressed_bytes,
            tile_reads_total,
            tile_writes_total,
            tile_deletes_total,
            operation_errors_total,
            tile_read_latency,
            tile_write_latency,
            query_latency,
            cache_hits_total,
            cache_misses_total,
            cache_size_bytes,
            memory_entries_total,
            memory_reads_total,
            memory_writes_total,
            revision_commits_total,
            revision_checkouts_total,
            gc_runs_total,
            gc_tiles_freed,
            gc_duration_seconds,
            recovery_runs_total,
            recovery_success_total,
            recovery_failure_total,
            recovery_duration_seconds,
            active_operations,
            compute_budget_remaining,
        })
    }

    /// Record tile read
    pub fn record_tile_read(&self, latency_seconds: f64) {
        self.tile_reads_total.inc();
        self.tile_read_latency.observe(latency_seconds);
    }

    /// Record tile write
    pub fn record_tile_write(&self, latency_seconds: f64) {
        self.tile_writes_total.inc();
        self.tile_write_latency.observe(latency_seconds);
    }

    /// Record tile delete
    pub fn record_tile_delete(&self) {
        self.tile_deletes_total.inc();
    }

    /// Record operation error
    pub fn record_operation_error(&self, error_type: &str) {
        self.operation_errors_total.with_label_values(&[error_type]).inc();
    }

    /// Record cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits_total.inc();
    }

    /// Record cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses_total.inc();
    }

    /// Record query
    pub fn record_query(&self, latency_seconds: f64) {
        self.query_latency.observe(latency_seconds);
    }

    /// Record revision commit
    pub fn record_revision_commit(&self) {
        self.revision_commits_total.inc();
    }

    /// Record revision checkout
    pub fn record_revision_checkout(&self) {
        self.revision_checkouts_total.inc();
    }

    /// Record GC run
    pub fn record_gc_run(&self, duration_seconds: f64, tiles_freed: u64) {
        self.gc_runs_total.inc();
        self.gc_tiles_freed.inc_by(tiles_freed as f64);
        self.gc_duration_seconds.observe(duration_seconds);
    }

    /// Record recovery
    pub fn record_recovery(&self, duration_seconds: f64, success: bool) {
        self.recovery_runs_total.inc();
        if success {
            self.recovery_success_total.inc();
        } else {
            self.recovery_failure_total.inc();
        }
        self.recovery_duration_seconds.observe(duration_seconds);
    }

    /// Update store metrics
    pub fn update_store_metrics(&self, tiles: u64, segments: u32, size: u64, compressed: u64) {
        self.store_tiles_total.set(tiles as f64);
        self.store_segments_total.set(segments as f64);
        self.store_size_bytes.set(size as f64);
        self.store_compressed_bytes.set(compressed as f64);
    }

    /// Update active operations
    pub fn set_active_operations(&self, count: u64) {
        self.active_operations.set(count as f64);
    }

    /// Update compute budget
    pub fn set_compute_budget(&self, remaining: u64) {
        self.compute_budget_remaining.set(remaining as f64);
    }

    /// Export metrics in Prometheus format
    pub fn export(&self) -> Result<String> {
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        encoder.encode(&self.registry.gather(), &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    /// Get registry
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}

impl Default for CnwsMetrics {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = CnwsMetrics::new().unwrap();
        metrics.record_tile_read(0.001);
        metrics.record_tile_write(0.002);
        metrics.record_cache_hit();
        metrics.record_revision_commit();

        let export = metrics.export().unwrap();
        assert!(export.contains("cnws_tile_reads_total"));
        assert!(export.contains("cnws_tile_writes_total"));
    }
}

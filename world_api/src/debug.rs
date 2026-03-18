use bevy::prelude::*;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::fmt::Write as _;
use std::time::Duration;

/// Shared resource — stan wyświetlany przez debug panel.
/// Wypełniany przez collect_debug_entries system w ui crate.
/// Każdy system w projekcie może pisać przez DebugEntry event.
#[derive(Resource, Default)]
pub struct DebugMetrics {
    pub sections: BTreeMap<&'static str, BTreeMap<&'static str, String>>,
}

impl DebugMetrics {
    pub fn set(&mut self, section: &'static str, key: &'static str, value: impl Display) {
        let entry = self
            .sections
            .entry(section)
            .or_default()
            .entry(key)
            .or_default();
        entry.clear();
        write!(entry, "{}", value).unwrap();
    }

    pub fn clear(&mut self) {
        self.sections.clear();
    }
}

/// Event — jeden wpis do debug panelu.
/// Każdy system emituje DebugEntry zamiast pisać bezpośrednio do DebugMetrics.
#[derive(Message, Clone)]
pub struct DebugEntry {
    pub section: &'static str,
    pub key: &'static str,
    pub value: String,
}

impl DebugEntry {
    pub fn new(section: &'static str, key: &'static str, value: impl Display) -> Self {
        Self {
            section,
            key,
            value: value.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SampleStats {
    last: f64,
    min: f64,
    max: f64,
    total: f64,
    samples: u64,
}

impl Default for SampleStats {
    fn default() -> Self {
        Self {
            last: 0.0,
            min: f64::INFINITY,
            max: 0.0,
            total: 0.0,
            samples: 0,
        }
    }
}

impl SampleStats {
    pub fn record(&mut self, value: f64) {
        self.last = value;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
        self.total += value;
        self.samples += 1;
    }

    pub fn record_duration(&mut self, duration: Duration) {
        self.record(duration.as_secs_f64() * 1_000.0);
    }

    pub fn is_empty(&self) -> bool {
        self.samples == 0
    }

    pub fn last(&self) -> Option<f64> {
        (!self.is_empty()).then_some(self.last)
    }

    pub fn min(&self) -> Option<f64> {
        (!self.is_empty()).then_some(self.min)
    }

    pub fn max(&self) -> Option<f64> {
        (!self.is_empty()).then_some(self.max)
    }

    pub fn avg(&self) -> Option<f64> {
        (!self.is_empty()).then_some(self.total / self.samples as f64)
    }

    pub fn total(&self) -> f64 {
        self.total
    }

    pub fn samples(&self) -> u64 {
        self.samples
    }

    pub fn format_summary(&self, decimals: usize, unit: &str) -> String {
        let (Some(last), Some(avg), Some(max)) = (self.last(), self.avg(), self.max()) else {
            return "n/a".to_string();
        };

        format!(
            "last {last:.decimals$}{unit} | avg {avg:.decimals$}{unit} | max {max:.decimals$}{unit}",
            decimals = decimals,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::SampleStats;
    use std::time::Duration;

    #[test]
    fn sample_stats_tracks_basic_aggregates() {
        let mut stats = SampleStats::default();
        stats.record(4.0);
        stats.record(8.0);
        stats.record_duration(Duration::from_millis(12));

        assert_eq!(stats.samples(), 3);
        assert_eq!(stats.last(), Some(12.0));
        assert_eq!(stats.min(), Some(4.0));
        assert_eq!(stats.max(), Some(12.0));
        assert_eq!(stats.avg(), Some(8.0));
        assert_eq!(stats.total(), 24.0);
    }

    #[test]
    fn sample_stats_formats_missing_and_present_values() {
        let mut stats = SampleStats::default();
        assert_eq!(stats.format_summary(2, " ms"), "n/a");

        stats.record(1.234);
        assert_eq!(
            stats.format_summary(2, " ms"),
            "last 1.23 ms | avg 1.23 ms | max 1.23 ms"
        );
    }
}

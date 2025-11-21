//! Request timing measurement and formatting utilities.
//!
//! This module provides functionality to measure detailed timing breakdown
//! of HTTP requests, including DNS lookup, TCP connection, TLS handshake,
//! time to first byte, and download time.

use crate::models::response::RequestTiming;
use std::time::{Duration, Instant};

/// Timing checkpoints collected during request execution.
///
/// Due to limitations in the Zed HTTP client API, not all timing phases
/// can be measured precisely. This struct tracks what we can measure and
/// estimates the rest based on total duration.
#[derive(Debug, Clone)]
pub struct TimingCheckpoints {
    /// When the request started.
    pub request_start: Instant,

    /// When the HTTP client began processing (after validation).
    pub client_start: Option<Instant>,

    /// When the request was sent to the network.
    pub request_sent: Option<Instant>,

    /// When the first response byte was received.
    pub first_byte_received: Option<Instant>,

    /// When the response was completely received.
    pub response_complete: Instant,

    /// Whether the request used HTTPS.
    pub is_https: bool,
}

impl TimingCheckpoints {
    /// Creates a new TimingCheckpoints with the request start time.
    ///
    /// # Arguments
    ///
    /// * `is_https` - Whether the request uses HTTPS
    ///
    /// # Returns
    ///
    /// A new `TimingCheckpoints` instance with start time recorded.
    pub fn new(is_https: bool) -> Self {
        Self {
            request_start: Instant::now(),
            client_start: None,
            request_sent: None,
            first_byte_received: None,
            response_complete: Instant::now(), // Will be updated
            is_https,
        }
    }

    /// Records when the HTTP client started processing the request.
    pub fn mark_client_start(&mut self) {
        self.client_start = Some(Instant::now());
    }

    /// Records when the request was sent to the network.
    pub fn mark_request_sent(&mut self) {
        self.request_sent = Some(Instant::now());
    }

    /// Records when the first response byte was received.
    pub fn mark_first_byte_received(&mut self) {
        self.first_byte_received = Some(Instant::now());
    }

    /// Records when the response was completely received.
    pub fn mark_response_complete(&mut self) {
        self.response_complete = Instant::now();
    }

    /// Converts checkpoints into a RequestTiming with estimated phase durations.
    ///
    /// Due to API limitations, we estimate timing phases:
    /// - DNS + TCP + (optional TLS): Time from start to request sent
    /// - First Byte: Time from request sent to first byte received
    /// - Download: Time from first byte to response complete
    ///
    /// # Returns
    ///
    /// A `RequestTiming` struct with duration breakdowns.
    pub fn to_request_timing(&self) -> RequestTiming {
        let total_duration = self.response_complete.duration_since(self.request_start);

        // If we have detailed checkpoints, use them
        if let (Some(request_sent), Some(first_byte)) =
            (self.request_sent, self.first_byte_received)
        {
            let connection_phase = request_sent.duration_since(self.request_start);
            let first_byte_duration = first_byte.duration_since(request_sent);
            let download_duration = self.response_complete.duration_since(first_byte);

            // Estimate DNS, TCP, and TLS breakdown from connection phase
            self.estimate_connection_phases(
                connection_phase,
                first_byte_duration,
                download_duration,
            )
        } else {
            // Fallback: Estimate all phases from total duration
            self.estimate_all_phases(total_duration)
        }
    }

    /// Estimates connection phases (DNS, TCP, TLS) from total connection time.
    fn estimate_connection_phases(
        &self,
        connection_phase: Duration,
        first_byte_duration: Duration,
        download_duration: Duration,
    ) -> RequestTiming {
        // Estimate phase breakdown based on typical network behavior:
        // DNS: ~15% of connection time
        // TCP: ~25% of connection time
        // TLS: ~60% of connection time (if HTTPS)

        if self.is_https {
            let dns_estimate = connection_phase.mul_f64(0.15);
            let tcp_estimate = connection_phase.mul_f64(0.25);
            let tls_estimate = connection_phase.mul_f64(0.60);

            RequestTiming {
                dns_lookup: dns_estimate,
                tcp_connection: tcp_estimate,
                tls_handshake: Some(tls_estimate),
                first_byte: first_byte_duration,
                download: download_duration,
            }
        } else {
            // HTTP: Split connection time between DNS and TCP
            let dns_estimate = connection_phase.mul_f64(0.40);
            let tcp_estimate = connection_phase.mul_f64(0.60);

            RequestTiming {
                dns_lookup: dns_estimate,
                tcp_connection: tcp_estimate,
                tls_handshake: None,
                first_byte: first_byte_duration,
                download: download_duration,
            }
        }
    }

    /// Estimates all timing phases from total duration when detailed checkpoints aren't available.
    fn estimate_all_phases(&self, total_duration: Duration) -> RequestTiming {
        // Estimate phase breakdown from total time:
        // Connection setup: ~40% (DNS + TCP + TLS)
        // Server processing: ~30% (first byte)
        // Download: ~30%

        let connection_time = total_duration.mul_f64(0.40);
        let first_byte_time = total_duration.mul_f64(0.30);
        let download_time = total_duration.mul_f64(0.30);

        if self.is_https {
            RequestTiming {
                dns_lookup: connection_time.mul_f64(0.15),
                tcp_connection: connection_time.mul_f64(0.25),
                tls_handshake: Some(connection_time.mul_f64(0.60)),
                first_byte: first_byte_time,
                download: download_time,
            }
        } else {
            RequestTiming {
                dns_lookup: connection_time.mul_f64(0.40),
                tcp_connection: connection_time.mul_f64(0.60),
                tls_handshake: None,
                first_byte: first_byte_time,
                download: download_time,
            }
        }
    }
}

/// Formats a timing breakdown into a human-readable string.
///
/// # Arguments
///
/// * `timing` - The request timing to format
///
/// # Returns
///
/// A formatted string like "DNS: 10ms | TCP: 20ms | TLS: 50ms | First Byte: 30ms | Download: 100ms"
///
/// # Examples
///
/// ```
/// use rest_client::executor::timing::format_timing_breakdown;
/// use rest_client::models::response::RequestTiming;
/// use std::time::Duration;
///
/// let mut timing = RequestTiming::new();
/// timing.dns_lookup = Duration::from_millis(10);
/// timing.tcp_connection = Duration::from_millis(20);
/// timing.tls_handshake = Some(Duration::from_millis(50));
/// timing.first_byte = Duration::from_millis(30);
/// timing.download = Duration::from_millis(100);
///
/// let formatted = format_timing_breakdown(&timing);
/// assert!(formatted.contains("DNS: 10ms"));
/// assert!(formatted.contains("TLS: 50ms"));
/// ```
pub fn format_timing_breakdown(timing: &RequestTiming) -> String {
    let dns = format_duration_human(&timing.dns_lookup);
    let tcp = format_duration_human(&timing.tcp_connection);
    let first_byte = format_duration_human(&timing.first_byte);
    let download = format_duration_human(&timing.download);

    if let Some(tls) = timing.tls_handshake {
        let tls_str = format_duration_human(&tls);
        format!(
            "DNS: {} | TCP: {} | TLS: {} | First Byte: {} | Download: {}",
            dns, tcp, tls_str, first_byte, download
        )
    } else {
        format!(
            "DNS: {} | TCP: {} | First Byte: {} | Download: {}",
            dns, tcp, first_byte, download
        )
    }
}

/// Formats a timing breakdown into a compact string for inline display.
///
/// # Arguments
///
/// * `timing` - The request timing to format
///
/// # Returns
///
/// A compact string like "DNS 10ms + TCP 20ms + TLS 50ms + TTFB 30ms + DL 100ms"
pub fn format_timing_compact(timing: &RequestTiming) -> String {
    let dns = format_duration_compact(&timing.dns_lookup);
    let tcp = format_duration_compact(&timing.tcp_connection);
    let first_byte = format_duration_compact(&timing.first_byte);
    let download = format_duration_compact(&timing.download);

    if let Some(tls) = timing.tls_handshake {
        let tls_str = format_duration_compact(&tls);
        format!(
            "DNS {} + TCP {} + TLS {} + TTFB {} + DL {}",
            dns, tcp, tls_str, first_byte, download
        )
    } else {
        format!(
            "DNS {} + TCP {} + TTFB {} + DL {}",
            dns, tcp, first_byte, download
        )
    }
}

/// Formats a duration in human-readable format with appropriate unit.
///
/// # Arguments
///
/// * `duration` - The duration to format
///
/// # Returns
///
/// A string like "10ms", "1.234s", or "1.5μs"
fn format_duration_human(duration: &Duration) -> String {
    let micros = duration.as_micros();

    if micros == 0 {
        "0μs".to_string()
    } else if micros < 1000 {
        format!("{}μs", micros)
    } else if micros < 1_000_000 {
        let millis = duration.as_millis();
        format!("{}ms", millis)
    } else {
        let secs = duration.as_secs_f64();
        format!("{:.3}s", secs)
    }
}

/// Formats a duration in compact format (ms only).
///
/// # Arguments
///
/// * `duration` - The duration to format
///
/// # Returns
///
/// A string like "10ms" or "1234ms"
fn format_duration_compact(duration: &Duration) -> String {
    let millis = duration.as_millis();
    format!("{}ms", millis)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timing_checkpoints_new() {
        let checkpoints = TimingCheckpoints::new(true);
        assert!(checkpoints.is_https);
        assert!(checkpoints.client_start.is_none());
        assert!(checkpoints.request_sent.is_none());
        assert!(checkpoints.first_byte_received.is_none());
    }

    #[test]
    fn test_timing_checkpoints_mark() {
        let mut checkpoints = TimingCheckpoints::new(false);

        checkpoints.mark_client_start();
        assert!(checkpoints.client_start.is_some());

        checkpoints.mark_request_sent();
        assert!(checkpoints.request_sent.is_some());

        checkpoints.mark_first_byte_received();
        assert!(checkpoints.first_byte_received.is_some());

        checkpoints.mark_response_complete();
    }

    #[test]
    fn test_to_request_timing_https() {
        let mut checkpoints = TimingCheckpoints::new(true);

        std::thread::sleep(Duration::from_millis(10));
        checkpoints.mark_request_sent();

        std::thread::sleep(Duration::from_millis(10));
        checkpoints.mark_first_byte_received();

        std::thread::sleep(Duration::from_millis(10));
        checkpoints.mark_response_complete();

        let timing = checkpoints.to_request_timing();

        // Should have TLS for HTTPS
        assert!(timing.tls_handshake.is_some());

        // Total should be sum of parts (with small margin for precision)
        let total = timing.total();
        assert!(total.as_millis() >= 30);
    }

    #[test]
    fn test_to_request_timing_http() {
        let mut checkpoints = TimingCheckpoints::new(false);

        std::thread::sleep(Duration::from_millis(10));
        checkpoints.mark_request_sent();

        std::thread::sleep(Duration::from_millis(10));
        checkpoints.mark_first_byte_received();

        std::thread::sleep(Duration::from_millis(10));
        checkpoints.mark_response_complete();

        let timing = checkpoints.to_request_timing();

        // Should NOT have TLS for HTTP
        assert!(timing.tls_handshake.is_none());
    }

    #[test]
    fn test_to_request_timing_fallback() {
        let mut checkpoints = TimingCheckpoints::new(true);

        std::thread::sleep(Duration::from_millis(30));
        checkpoints.mark_response_complete();

        let timing = checkpoints.to_request_timing();

        // Should still produce valid timing even without detailed checkpoints
        assert!(timing.dns_lookup.as_millis() > 0);
        assert!(timing.tcp_connection.as_millis() > 0);
        assert!(timing.tls_handshake.is_some());
        assert!(timing.first_byte.as_millis() > 0);
        assert!(timing.download.as_millis() > 0);
    }

    #[test]
    fn test_format_timing_breakdown_https() {
        let timing = RequestTiming {
            dns_lookup: Duration::from_millis(10),
            tcp_connection: Duration::from_millis(20),
            tls_handshake: Some(Duration::from_millis(50)),
            first_byte: Duration::from_millis(30),
            download: Duration::from_millis(100),
        };

        let formatted = format_timing_breakdown(&timing);
        assert!(formatted.contains("DNS: 10ms"));
        assert!(formatted.contains("TCP: 20ms"));
        assert!(formatted.contains("TLS: 50ms"));
        assert!(formatted.contains("First Byte: 30ms"));
        assert!(formatted.contains("Download: 100ms"));
    }

    #[test]
    fn test_format_timing_breakdown_http() {
        let timing = RequestTiming {
            dns_lookup: Duration::from_millis(10),
            tcp_connection: Duration::from_millis(20),
            tls_handshake: None,
            first_byte: Duration::from_millis(30),
            download: Duration::from_millis(100),
        };

        let formatted = format_timing_breakdown(&timing);
        assert!(formatted.contains("DNS: 10ms"));
        assert!(formatted.contains("TCP: 20ms"));
        assert!(!formatted.contains("TLS"));
        assert!(formatted.contains("First Byte: 30ms"));
        assert!(formatted.contains("Download: 100ms"));
    }

    #[test]
    fn test_format_timing_compact() {
        let timing = RequestTiming {
            dns_lookup: Duration::from_millis(10),
            tcp_connection: Duration::from_millis(20),
            tls_handshake: Some(Duration::from_millis(50)),
            first_byte: Duration::from_millis(30),
            download: Duration::from_millis(100),
        };

        let formatted = format_timing_compact(&timing);
        assert!(formatted.contains("DNS 10ms"));
        assert!(formatted.contains("TCP 20ms"));
        assert!(formatted.contains("TLS 50ms"));
        assert!(formatted.contains("TTFB 30ms"));
        assert!(formatted.contains("DL 100ms"));
    }

    #[test]
    fn test_format_duration_human() {
        assert_eq!(format_duration_human(&Duration::from_micros(0)), "0μs");
        assert_eq!(format_duration_human(&Duration::from_micros(500)), "500μs");
        assert_eq!(format_duration_human(&Duration::from_millis(10)), "10ms");
        assert_eq!(format_duration_human(&Duration::from_millis(999)), "999ms");
        assert_eq!(format_duration_human(&Duration::from_secs(1)), "1.000s");
        assert_eq!(
            format_duration_human(&Duration::from_millis(1234)),
            "1.234s"
        );
    }

    #[test]
    fn test_format_duration_compact() {
        assert_eq!(format_duration_compact(&Duration::from_micros(500)), "0ms");
        assert_eq!(format_duration_compact(&Duration::from_millis(10)), "10ms");
        assert_eq!(format_duration_compact(&Duration::from_secs(1)), "1000ms");
    }

    #[test]
    fn test_timing_breakdown_sum_equals_total() {
        let timing = RequestTiming {
            dns_lookup: Duration::from_millis(10),
            tcp_connection: Duration::from_millis(20),
            tls_handshake: Some(Duration::from_millis(50)),
            first_byte: Duration::from_millis(30),
            download: Duration::from_millis(100),
        };

        let total = timing.total();
        assert_eq!(total, Duration::from_millis(210));
    }
}

//! Progress reporting utilities for batch operations.
//!
//! Provides reusable progress tracking for parallel game execution
//! in arena, validate, and tune binaries.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// Style of progress output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressStyle {
    /// Simple progress: "Progress: 50% (500/1000)"
    Simple,
    /// Rich progress with rate and ETA: "Progress: 50% (500/1000) | 100 games/sec | ETA: 5.0s"
    Rich,
}

/// Reusable progress reporter for batch operations.
///
/// Spawns a background thread that periodically updates stderr with progress.
/// Thread-safe counter can be incremented from parallel worker threads.
pub struct ProgressReporter {
    total: usize,
    completed: Arc<AtomicUsize>,
    stop_flag: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    start_time: Instant,
    style: ProgressStyle,
    prefix: String,
}

impl ProgressReporter {
    /// Create a new progress reporter.
    ///
    /// # Arguments
    /// * `total` - Total number of items to process
    pub fn new(total: usize) -> Self {
        Self {
            total,
            completed: Arc::new(AtomicUsize::new(0)),
            stop_flag: Arc::new(AtomicBool::new(false)),
            handle: None,
            start_time: Instant::now(),
            style: ProgressStyle::Rich,
            prefix: String::new(),
        }
    }

    /// Set the progress style.
    pub fn with_style(mut self, style: ProgressStyle) -> Self {
        self.style = style;
        self
    }

    /// Set a prefix for progress output (e.g., "  " for indentation).
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// Start the progress reporting background thread.
    ///
    /// Call this before starting parallel work. The thread will automatically
    /// stop when `completed >= total`, when `finish()` is called, or when
    /// `finish_silent()` signals cancellation.
    pub fn start(mut self) -> Self {
        let total = self.total;
        let completed = self.completed.clone();
        let stop_flag = self.stop_flag.clone();
        let start_time = self.start_time;
        let style = self.style;
        let prefix = self.prefix.clone();

        let handle = thread::spawn(move || {
            let mut last_progress = 0;
            loop {
                thread::sleep(Duration::from_millis(100));
                // Check stop flag first
                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }
                let done = completed.load(Ordering::Relaxed);
                if done >= total {
                    break;
                }
                let progress = if total > 0 { (done * 100) / total } else { 0 };

                match style {
                    ProgressStyle::Simple => {
                        eprint!("\r{}Progress: {:3}% ({}/{})    ", prefix, progress, done, total);
                    }
                    ProgressStyle::Rich => {
                        if progress > last_progress {
                            let elapsed = start_time.elapsed().as_secs_f64();
                            let rate = done as f64 / elapsed.max(0.001);
                            let eta = if rate > 0.0 {
                                (total - done) as f64 / rate
                            } else {
                                0.0
                            };
                            eprint!(
                                "\r{}Progress: {:3}% ({}/{}) | {:.0} games/sec | ETA: {:.1}s    ",
                                prefix, progress, done, total, rate, eta
                            );
                            last_progress = progress;
                        }
                    }
                }
            }
        });

        self.handle = Some(handle);
        self
    }

    /// Get a clone of the counter for worker threads to increment.
    pub fn counter(&self) -> Arc<AtomicUsize> {
        self.completed.clone()
    }

    /// Increment the completed count by one.
    ///
    /// This is a convenience method; you can also call `fetch_add` on the counter directly.
    #[inline]
    pub fn increment(&self) {
        self.completed.fetch_add(1, Ordering::Relaxed);
    }

    /// Get the current completed count.
    pub fn completed(&self) -> usize {
        self.completed.load(Ordering::Relaxed)
    }

    /// Finish progress reporting and print final status.
    ///
    /// Waits for the background thread to complete and prints a final 100% line.
    pub fn finish(self) {
        if let Some(handle) = self.handle {
            let _ = handle.join();
        }
        let done = self.completed.load(Ordering::Relaxed);
        match self.style {
            ProgressStyle::Simple => {
                eprintln!(
                    "\r{}Progress: 100% ({}/{})    ",
                    self.prefix, done, self.total
                );
            }
            ProgressStyle::Rich => {
                let elapsed = self.start_time.elapsed().as_secs_f64();
                let rate = done as f64 / elapsed.max(0.001);
                eprintln!(
                    "\r{}Progress: 100% ({}/{}) | {:.0} games/sec | Done in {:.1}s    ",
                    self.prefix, done, self.total, rate, elapsed
                );
            }
        }
    }

    /// Finish without printing final status (silent finish).
    ///
    /// Signals the background thread to stop and waits for it to complete.
    pub fn finish_silent(self) {
        // Signal the thread to stop
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle {
            let _ = handle.join();
        }
    }
}

/// Helper to conditionally create and start a progress reporter.
///
/// Returns `None` if `show_progress` is false, otherwise returns a started reporter.
pub fn maybe_progress(
    show_progress: bool,
    total: usize,
    style: ProgressStyle,
) -> Option<ProgressReporter> {
    if show_progress {
        Some(ProgressReporter::new(total).with_style(style).start())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_counter() {
        let reporter = ProgressReporter::new(100);
        assert_eq!(reporter.completed(), 0);

        reporter.increment();
        assert_eq!(reporter.completed(), 1);

        reporter.increment();
        reporter.increment();
        assert_eq!(reporter.completed(), 3);
    }

    #[test]
    fn test_progress_counter_arc() {
        let reporter = ProgressReporter::new(100);
        let counter = reporter.counter();

        counter.fetch_add(5, Ordering::Relaxed);
        assert_eq!(reporter.completed(), 5);
    }

    #[test]
    fn test_maybe_progress_disabled() {
        let reporter = maybe_progress(false, 100, ProgressStyle::Simple);
        assert!(reporter.is_none());
    }

    #[test]
    fn test_maybe_progress_enabled() {
        let reporter = maybe_progress(true, 100, ProgressStyle::Simple);
        assert!(reporter.is_some());
        // Clean up without printing
        reporter.unwrap().finish_silent();
    }

    #[test]
    fn test_progress_with_prefix() {
        let reporter = ProgressReporter::new(100).with_prefix("  ");
        assert_eq!(reporter.prefix, "  ");
    }

    #[test]
    fn test_progress_styles() {
        let simple = ProgressReporter::new(100).with_style(ProgressStyle::Simple);
        assert_eq!(simple.style, ProgressStyle::Simple);

        let rich = ProgressReporter::new(100).with_style(ProgressStyle::Rich);
        assert_eq!(rich.style, ProgressStyle::Rich);
    }
}

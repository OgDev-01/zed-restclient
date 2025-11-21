//! Request cancellation tracking and management.
//!
//! This module provides infrastructure for canceling in-flight HTTP requests.
//! It uses tokio task handles to track active requests and allows them to be
//! aborted on demand.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// A handle to a running request that can be cancelled.
#[derive(Debug)]
pub struct RequestHandle {
    /// Unique identifier for this request.
    pub request_id: String,

    /// The tokio task handle for the executing request.
    /// Note: In the current Zed extension context (WASM), we can't use tokio::task::JoinHandle
    /// because tokio is only available in dev-dependencies. This is a placeholder for when
    /// proper async cancellation is supported by the Zed extension API.
    #[cfg(test)]
    pub join_handle: Option<tokio::task::JoinHandle<()>>,

    /// Flag to mark if cancellation was requested.
    /// This is used in the WASM context where we can't use tokio JoinHandles.
    pub cancelled: Arc<Mutex<bool>>,
}

impl RequestHandle {
    /// Creates a new request handle with a generated UUID.
    pub fn new() -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            #[cfg(test)]
            join_handle: None,
            cancelled: Arc::new(Mutex::new(false)),
        }
    }

    /// Creates a new request handle with a specific request ID.
    pub fn with_id(request_id: String) -> Self {
        Self {
            request_id,
            #[cfg(test)]
            join_handle: None,
            cancelled: Arc::new(Mutex::new(false)),
        }
    }

    /// Checks if cancellation has been requested for this request.
    pub fn is_cancelled(&self) -> bool {
        *self.cancelled.lock().unwrap()
    }

    /// Marks this request as cancelled.
    pub fn mark_cancelled(&self) {
        *self.cancelled.lock().unwrap() = true;
    }
}

impl Default for RequestHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// Error types for cancellation operations.
#[derive(Debug, Clone, PartialEq)]
pub enum CancelError {
    /// Request with the given ID was not found.
    NotFound(String),

    /// Request has already completed and cannot be cancelled.
    AlreadyCompleted(String),

    /// Failed to acquire lock on tracker.
    LockError(String),
}

impl std::fmt::Display for CancelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CancelError::NotFound(id) => {
                write!(f, "Request not found: {}", id)
            }
            CancelError::AlreadyCompleted(id) => {
                write!(f, "Request already completed: {}", id)
            }
            CancelError::LockError(msg) => {
                write!(f, "Lock error: {}", msg)
            }
        }
    }
}

impl std::error::Error for CancelError {}

/// Tracks active HTTP requests and manages cancellation.
///
/// This struct maintains a registry of all in-flight requests and provides
/// methods to cancel them. It's thread-safe and can be shared across tasks.
#[derive(Debug, Default)]
pub struct RequestTracker {
    /// Map of request IDs to their handles.
    active_requests: HashMap<String, RequestHandle>,

    /// Order of request IDs by insertion time (oldest first).
    request_order: Vec<String>,
}

impl RequestTracker {
    /// Creates a new empty request tracker.
    pub fn new() -> Self {
        Self {
            active_requests: HashMap::new(),
            request_order: Vec::new(),
        }
    }

    /// Registers a new request for tracking.
    ///
    /// # Arguments
    ///
    /// * `handle` - The request handle to track
    ///
    /// # Returns
    ///
    /// The request ID of the registered request.
    pub fn register(&mut self, handle: RequestHandle) -> String {
        let request_id = handle.request_id.clone();
        self.request_order.push(request_id.clone());
        self.active_requests.insert(request_id.clone(), handle);
        request_id
    }

    /// Removes a request from tracking (called when request completes).
    ///
    /// # Arguments
    ///
    /// * `request_id` - ID of the request to remove
    ///
    /// # Returns
    ///
    /// `true` if the request was found and removed, `false` otherwise.
    pub fn unregister(&mut self, request_id: &str) -> bool {
        // Remove from order tracking
        if let Some(pos) = self.request_order.iter().position(|id| id == request_id) {
            self.request_order.remove(pos);
        }

        // Remove from active requests
        self.active_requests.remove(request_id).is_some()
    }

    /// Cancels a specific request by ID.
    ///
    /// # Arguments
    ///
    /// * `request_id` - ID of the request to cancel
    ///
    /// # Returns
    ///
    /// `Ok(())` if cancellation was successful, or `Err(CancelError)` if the
    /// request was not found or already completed.
    pub fn cancel_request(&mut self, request_id: &str) -> Result<(), CancelError> {
        let handle = self
            .active_requests
            .get(request_id)
            .ok_or_else(|| CancelError::NotFound(request_id.to_string()))?;

        // Mark as cancelled
        handle.mark_cancelled();

        // In a full tokio environment, we would call handle.join_handle.abort() here
        #[cfg(test)]
        if let Some(ref join_handle) = handle.join_handle {
            join_handle.abort();
        }

        // Remove from tracking
        self.unregister(request_id);

        Ok(())
    }

    /// Cancels the most recently started request.
    ///
    /// # Returns
    ///
    /// `Ok(request_id)` if a request was cancelled, or `Err(CancelError::NotFound)`
    /// if there are no active requests.
    pub fn cancel_most_recent(&mut self) -> Result<String, CancelError> {
        let request_id = self
            .request_order
            .last()
            .ok_or_else(|| CancelError::NotFound("no active requests".to_string()))?
            .clone();

        self.cancel_request(&request_id)?;
        Ok(request_id)
    }

    /// Gets the number of active requests currently being tracked.
    pub fn active_count(&self) -> usize {
        self.active_requests.len()
    }

    /// Gets a list of all active request IDs.
    pub fn active_request_ids(&self) -> Vec<String> {
        self.request_order.clone()
    }

    /// Checks if a specific request is still active.
    pub fn is_active(&self, request_id: &str) -> bool {
        self.active_requests.contains_key(request_id)
    }

    /// Cleans up completed requests (removes those marked as cancelled).
    ///
    /// This is useful for preventing memory leaks in long-running sessions.
    ///
    /// # Returns
    ///
    /// Number of requests that were cleaned up.
    pub fn cleanup_completed(&mut self) -> usize {
        let completed_ids: Vec<String> = self
            .active_requests
            .iter()
            .filter(|(_, handle)| handle.is_cancelled())
            .map(|(id, _)| id.clone())
            .collect();

        let count = completed_ids.len();
        for id in completed_ids {
            self.unregister(&id);
        }

        count
    }
}

/// Thread-safe wrapper around RequestTracker.
///
/// This allows the tracker to be shared across threads and tasks safely.
#[derive(Debug, Clone)]
pub struct SharedRequestTracker {
    inner: Arc<Mutex<RequestTracker>>,
}

impl SharedRequestTracker {
    /// Creates a new shared request tracker.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(RequestTracker::new())),
        }
    }

    /// Registers a new request for tracking.
    pub fn register(&self, handle: RequestHandle) -> Result<String, CancelError> {
        let mut tracker = self
            .inner
            .lock()
            .map_err(|e| CancelError::LockError(e.to_string()))?;
        Ok(tracker.register(handle))
    }

    /// Removes a request from tracking.
    pub fn unregister(&self, request_id: &str) -> Result<bool, CancelError> {
        let mut tracker = self
            .inner
            .lock()
            .map_err(|e| CancelError::LockError(e.to_string()))?;
        Ok(tracker.unregister(request_id))
    }

    /// Cancels a specific request.
    pub fn cancel_request(&self, request_id: &str) -> Result<(), CancelError> {
        let mut tracker = self
            .inner
            .lock()
            .map_err(|e| CancelError::LockError(e.to_string()))?;
        tracker.cancel_request(request_id)
    }

    /// Cancels the most recent request.
    pub fn cancel_most_recent(&self) -> Result<String, CancelError> {
        let mut tracker = self
            .inner
            .lock()
            .map_err(|e| CancelError::LockError(e.to_string()))?;
        tracker.cancel_most_recent()
    }

    /// Gets the count of active requests.
    pub fn active_count(&self) -> Result<usize, CancelError> {
        let tracker = self
            .inner
            .lock()
            .map_err(|e| CancelError::LockError(e.to_string()))?;
        Ok(tracker.active_count())
    }

    /// Gets all active request IDs.
    pub fn active_request_ids(&self) -> Result<Vec<String>, CancelError> {
        let tracker = self
            .inner
            .lock()
            .map_err(|e| CancelError::LockError(e.to_string()))?;
        Ok(tracker.active_request_ids())
    }

    /// Checks if a request is active.
    pub fn is_active(&self, request_id: &str) -> Result<bool, CancelError> {
        let tracker = self
            .inner
            .lock()
            .map_err(|e| CancelError::LockError(e.to_string()))?;
        Ok(tracker.is_active(request_id))
    }

    /// Cleans up completed requests.
    pub fn cleanup_completed(&self) -> Result<usize, CancelError> {
        let mut tracker = self
            .inner
            .lock()
            .map_err(|e| CancelError::LockError(e.to_string()))?;
        Ok(tracker.cleanup_completed())
    }
}

impl Default for SharedRequestTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_handle_creation() {
        let handle = RequestHandle::new();
        assert!(!handle.request_id.is_empty());
        assert!(!handle.is_cancelled());
    }

    #[test]
    fn test_request_handle_with_id() {
        let handle = RequestHandle::with_id("test-123".to_string());
        assert_eq!(handle.request_id, "test-123");
        assert!(!handle.is_cancelled());
    }

    #[test]
    fn test_request_handle_cancellation() {
        let handle = RequestHandle::new();
        assert!(!handle.is_cancelled());

        handle.mark_cancelled();
        assert!(handle.is_cancelled());
    }

    #[test]
    fn test_tracker_register_and_unregister() {
        let mut tracker = RequestTracker::new();
        assert_eq!(tracker.active_count(), 0);

        let handle = RequestHandle::with_id("req-1".to_string());
        let id = tracker.register(handle);

        assert_eq!(id, "req-1");
        assert_eq!(tracker.active_count(), 1);
        assert!(tracker.is_active("req-1"));

        let removed = tracker.unregister("req-1");
        assert!(removed);
        assert_eq!(tracker.active_count(), 0);
        assert!(!tracker.is_active("req-1"));
    }

    #[test]
    fn test_tracker_cancel_request() {
        let mut tracker = RequestTracker::new();

        let handle = RequestHandle::with_id("req-1".to_string());
        tracker.register(handle);

        let result = tracker.cancel_request("req-1");
        assert!(result.is_ok());
        assert_eq!(tracker.active_count(), 0);
    }

    #[test]
    fn test_tracker_cancel_nonexistent() {
        let mut tracker = RequestTracker::new();

        let result = tracker.cancel_request("nonexistent");
        assert!(result.is_err());
        assert!(matches!(result, Err(CancelError::NotFound(_))));
    }

    #[test]
    fn test_tracker_cancel_most_recent() {
        let mut tracker = RequestTracker::new();

        let handle1 = RequestHandle::with_id("req-1".to_string());
        let handle2 = RequestHandle::with_id("req-2".to_string());
        let handle3 = RequestHandle::with_id("req-3".to_string());

        tracker.register(handle1);
        tracker.register(handle2);
        tracker.register(handle3);

        assert_eq!(tracker.active_count(), 3);

        let result = tracker.cancel_most_recent();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "req-3");
        assert_eq!(tracker.active_count(), 2);

        let result = tracker.cancel_most_recent();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "req-2");
        assert_eq!(tracker.active_count(), 1);
    }

    #[test]
    fn test_tracker_cancel_most_recent_empty() {
        let mut tracker = RequestTracker::new();

        let result = tracker.cancel_most_recent();
        assert!(result.is_err());
        assert!(matches!(result, Err(CancelError::NotFound(_))));
    }

    #[test]
    fn test_tracker_cleanup_completed() {
        let mut tracker = RequestTracker::new();

        let handle1 = RequestHandle::with_id("req-1".to_string());
        let handle2 = RequestHandle::with_id("req-2".to_string());
        let handle3 = RequestHandle::with_id("req-3".to_string());

        let cancelled_flag = handle2.cancelled.clone();

        tracker.register(handle1);
        tracker.register(handle2);
        tracker.register(handle3);

        // Mark req-2 as cancelled
        *cancelled_flag.lock().unwrap() = true;

        assert_eq!(tracker.active_count(), 3);

        let cleaned = tracker.cleanup_completed();
        assert_eq!(cleaned, 1);
        assert_eq!(tracker.active_count(), 2);
        assert!(!tracker.is_active("req-2"));
        assert!(tracker.is_active("req-1"));
        assert!(tracker.is_active("req-3"));
    }

    #[test]
    fn test_shared_tracker_thread_safety() {
        let tracker = SharedRequestTracker::new();

        let handle1 = RequestHandle::with_id("req-1".to_string());
        let handle2 = RequestHandle::with_id("req-2".to_string());

        let id1 = tracker.register(handle1).unwrap();
        let id2 = tracker.register(handle2).unwrap();

        assert_eq!(id1, "req-1");
        assert_eq!(id2, "req-2");
        assert_eq!(tracker.active_count().unwrap(), 2);

        tracker.cancel_request("req-1").unwrap();
        assert_eq!(tracker.active_count().unwrap(), 1);
        assert!(!tracker.is_active("req-1").unwrap());
        assert!(tracker.is_active("req-2").unwrap());
    }

    #[test]
    fn test_shared_tracker_cancel_most_recent() {
        let tracker = SharedRequestTracker::new();

        let handle1 = RequestHandle::with_id("req-1".to_string());
        let handle2 = RequestHandle::with_id("req-2".to_string());

        tracker.register(handle1).unwrap();
        tracker.register(handle2).unwrap();

        let cancelled_id = tracker.cancel_most_recent().unwrap();
        assert_eq!(cancelled_id, "req-2");
        assert_eq!(tracker.active_count().unwrap(), 1);
    }

    #[test]
    fn test_active_request_ids() {
        let mut tracker = RequestTracker::new();

        let handle1 = RequestHandle::with_id("req-1".to_string());
        let handle2 = RequestHandle::with_id("req-2".to_string());
        let handle3 = RequestHandle::with_id("req-3".to_string());

        tracker.register(handle1);
        tracker.register(handle2);
        tracker.register(handle3);

        let ids = tracker.active_request_ids();
        assert_eq!(ids.len(), 3);
        assert_eq!(ids, vec!["req-1", "req-2", "req-3"]);
    }

    #[test]
    fn test_cancel_error_display() {
        let err1 = CancelError::NotFound("req-123".to_string());
        assert_eq!(err1.to_string(), "Request not found: req-123");

        let err2 = CancelError::AlreadyCompleted("req-456".to_string());
        assert_eq!(err2.to_string(), "Request already completed: req-456");

        let err3 = CancelError::LockError("mutex poisoned".to_string());
        assert_eq!(err3.to_string(), "Lock error: mutex poisoned");
    }

    // Edge case tests for cancellation timing

    #[test]
    fn test_cancel_immediately_after_register() {
        // Tests race condition: cancel right after registration
        let mut tracker = RequestTracker::new();
        let handle = RequestHandle::with_id("immediate-cancel".to_string());

        tracker.register(handle);

        // Cancel immediately
        let result = tracker.cancel_request("immediate-cancel");
        assert!(result.is_ok());
        assert_eq!(tracker.active_count(), 0);
    }

    #[test]
    fn test_double_unregister() {
        // Tests idempotency: unregistering twice should be safe
        let mut tracker = RequestTracker::new();
        let handle = RequestHandle::with_id("double-unreg".to_string());

        tracker.register(handle);

        let first = tracker.unregister("double-unreg");
        assert!(first);

        let second = tracker.unregister("double-unreg");
        assert!(!second); // Already removed
    }

    #[test]
    fn test_cancel_while_another_completes() {
        // Tests concurrent completion and cancellation
        let mut tracker = RequestTracker::new();

        let handle1 = RequestHandle::with_id("req-1".to_string());
        let handle2 = RequestHandle::with_id("req-2".to_string());

        tracker.register(handle1);
        tracker.register(handle2);

        // Simulate req-1 completing naturally
        tracker.unregister("req-1");

        // Try to cancel req-2 (should succeed)
        let result = tracker.cancel_request("req-2");
        assert!(result.is_ok());

        // Try to cancel req-1 (should fail - already completed)
        let result = tracker.cancel_request("req-1");
        assert!(result.is_err());
    }

    #[test]
    fn test_rapid_register_cancel_cycle() {
        // Tests rapid registration and cancellation
        let mut tracker = RequestTracker::new();

        for i in 0..10 {
            let handle = RequestHandle::with_id(format!("req-{}", i));
            tracker.register(handle);
        }

        assert_eq!(tracker.active_count(), 10);

        // Cancel all in reverse order
        for i in (0..10).rev() {
            let result = tracker.cancel_request(&format!("req-{}", i));
            assert!(result.is_ok());
        }

        assert_eq!(tracker.active_count(), 0);
    }

    #[test]
    fn test_cancellation_flag_persistence() {
        // Tests that cancellation flag persists correctly
        let handle = RequestHandle::with_id("persist-test".to_string());
        let flag = handle.cancelled.clone();

        assert!(!handle.is_cancelled());
        assert!(!*flag.lock().unwrap());

        handle.mark_cancelled();

        assert!(handle.is_cancelled());
        assert!(*flag.lock().unwrap());

        // Flag should persist even after multiple checks
        assert!(handle.is_cancelled());
        assert!(handle.is_cancelled());
    }

    #[test]
    fn test_shared_tracker_concurrent_operations() {
        // Tests thread-safe operations on SharedRequestTracker
        let tracker = SharedRequestTracker::new();

        // Simulate concurrent registrations
        let handle1 = RequestHandle::with_id("concurrent-1".to_string());
        let handle2 = RequestHandle::with_id("concurrent-2".to_string());
        let handle3 = RequestHandle::with_id("concurrent-3".to_string());

        tracker.register(handle1).unwrap();
        tracker.register(handle2).unwrap();
        tracker.register(handle3).unwrap();

        assert_eq!(tracker.active_count().unwrap(), 3);

        // Cancel middle request
        tracker.cancel_request("concurrent-2").unwrap();

        assert_eq!(tracker.active_count().unwrap(), 2);
        assert!(tracker.is_active("concurrent-1").unwrap());
        assert!(!tracker.is_active("concurrent-2").unwrap());
        assert!(tracker.is_active("concurrent-3").unwrap());

        // Get active IDs and verify order is preserved (minus the cancelled one)
        let ids = tracker.active_request_ids().unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"concurrent-1".to_string()));
        assert!(ids.contains(&"concurrent-3".to_string()));
    }

    #[test]
    fn test_cleanup_with_mixed_states() {
        // Tests cleanup with some cancelled and some active requests
        let mut tracker = RequestTracker::new();

        let handle1 = RequestHandle::with_id("active-1".to_string());
        let handle2 = RequestHandle::with_id("cancelled-1".to_string());
        let handle3 = RequestHandle::with_id("active-2".to_string());
        let handle4 = RequestHandle::with_id("cancelled-2".to_string());

        let cancel_flag2 = handle2.cancelled.clone();
        let cancel_flag4 = handle4.cancelled.clone();

        tracker.register(handle1);
        tracker.register(handle2);
        tracker.register(handle3);
        tracker.register(handle4);

        // Mark some as cancelled
        *cancel_flag2.lock().unwrap() = true;
        *cancel_flag4.lock().unwrap() = true;

        assert_eq!(tracker.active_count(), 4);

        let cleaned = tracker.cleanup_completed();
        assert_eq!(cleaned, 2);
        assert_eq!(tracker.active_count(), 2);

        // Verify only the non-cancelled ones remain
        assert!(tracker.is_active("active-1"));
        assert!(!tracker.is_active("cancelled-1"));
        assert!(tracker.is_active("active-2"));
        assert!(!tracker.is_active("cancelled-2"));
    }

    #[test]
    fn test_request_order_preservation() {
        // Tests that request order is preserved for cancel_most_recent
        let mut tracker = RequestTracker::new();

        let handle1 = RequestHandle::with_id("first".to_string());
        let handle2 = RequestHandle::with_id("second".to_string());
        let handle3 = RequestHandle::with_id("third".to_string());

        tracker.register(handle1);
        tracker.register(handle2);
        tracker.register(handle3);

        // Most recent should be "third"
        let cancelled = tracker.cancel_most_recent().unwrap();
        assert_eq!(cancelled, "third");

        // Now most recent should be "second"
        let cancelled = tracker.cancel_most_recent().unwrap();
        assert_eq!(cancelled, "second");

        // Finally "first"
        let cancelled = tracker.cancel_most_recent().unwrap();
        assert_eq!(cancelled, "first");

        // No more requests
        let result = tracker.cancel_most_recent();
        assert!(result.is_err());
    }
}

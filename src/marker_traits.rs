//! Marker traits for execution contexts
//!
//! These traits help distinguish between synchronous and asynchronous execution
//! contexts and enable better type-level guarantees for flow orchestration.

/// Marker trait for synchronous execution context
pub trait SyncContext: Send {}

/// Marker trait for asynchronous execution context
pub trait AsyncContext: Send {}

// Implement the markers for common types
impl<T: Send> SyncContext for T {}
impl<T: Send> AsyncContext for T {}

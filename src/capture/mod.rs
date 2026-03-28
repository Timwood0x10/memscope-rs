//! Capture Engine - Event capture backend
//!
//! This module provides the CaptureEngine which is responsible for
//! capturing memory events from the application and forwarding them
//! to the EventStore. The CaptureEngine does not store events itself;
//! it only captures and forwards them.

pub mod backends;
pub mod platform;
pub mod engine;

pub use backends::{CaptureBackend, CaptureBackendType};
pub use engine::CaptureEngine;
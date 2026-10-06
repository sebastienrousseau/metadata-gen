// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Concurrency verification module using Loom.
//!
//! Provides harness configurations and utilities for verifying concurrent
//! synchronization primitives under all possible execution interleavings.
//!
//! # Configuration
//!
//! * `MAX_PREEMPTIONS`: The maximum number of thread preemptions Loom will explore.
//!   Bound to 3 for smoke testing to keep execution time under 5 minutes while
//!   providing robust concurrency coverage.

/// Maximum thread preemptions explored by Loom model checking.
pub const MAX_PREEMPTIONS: usize = 3;

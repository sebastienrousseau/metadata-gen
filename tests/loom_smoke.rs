// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Concurrency verification smoke test using Loom.
//!
//! Verifies synchronization patterns under all thread interleavings.
//!
//! To run under Loom:
//! ```sh
//! RUSTFLAGS="--cfg loom" cargo test --test loom_smoke --release
//! ```

#[path = "loom/mod.rs"]
mod loom_config;

#[cfg(loom)]
mod loom_tests {
    use super::loom_config::MAX_PREEMPTIONS;
    use loom::sync::atomic::AtomicUsize;
    use loom::sync::Arc;
    use loom::thread;
    use std::sync::atomic::Ordering;

    #[test]
    fn test_concurrent_smoke() {
        let mut builder = loom::model::Builder::new();
        builder.preemption_bound = Some(MAX_PREEMPTIONS);
        builder.check(|| {
            let counter = Arc::new(AtomicUsize::new(0));
            let counter_clone = counter.clone();

            let h = thread::spawn(move || {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            });

            counter.fetch_add(1, Ordering::SeqCst);
            h.join().unwrap();

            assert_eq!(counter.load(Ordering::SeqCst), 2);
        });
    }
}

#[cfg(not(loom))]
#[test]
fn test_loom_vacuous_smoke() {
    // Vacuous test to keep infrastructure warm when --cfg loom is not active.
    assert_eq!(loom_config::MAX_PREEMPTIONS, 3);
}

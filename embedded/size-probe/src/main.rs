// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Size probe: the smallest firmware image that parses YAML front
//! matter with metadata-gen. It is linked, never run.

#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::hint::black_box;
use core::panic::PanicInfo;
use core::ptr;

const ARENA_SIZE: usize = 16 * 1024;

const INPUT: &str = "---\ntitle: Sensor node\ndescription: Field unit\nkeywords: iot, yaml\n---\nbody";

/// A bump allocator over a static arena that never frees. The probe
/// is single-threaded, so it needs no atomics and no lock.
struct Bump {
    arena: UnsafeCell<[u8; ARENA_SIZE]>,
    used: UnsafeCell<usize>,
}

// SAFETY: the image runs one thread and no interrupt handler
// allocates, so the cells are never accessed concurrently.
unsafe impl Sync for Bump {}

// SAFETY: every returned block lies inside the arena, is aligned to
// `layout.align()`, and is never handed out twice.
unsafe impl GlobalAlloc for Bump {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: single-threaded, see `impl Sync`.
        let used = unsafe { &mut *self.used.get() };
        let base = self.arena.get() as usize;
        let mask = layout.align() - 1;
        let start = ((base + *used + mask) & !mask) - base;
        match start.checked_add(layout.size()) {
            Some(end) if end <= ARENA_SIZE => {
                *used = end;
                // SAFETY: `start + size <= ARENA_SIZE`, inside the arena.
                unsafe { self.arena.get().cast::<u8>().add(start) }
            }
            _ => ptr::null_mut(),
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static HEAP: Bump = Bump {
    arena: UnsafeCell::new([0; ARENA_SIZE]),
    used: UnsafeCell::new(0),
};

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

/// The baseline: a heap allocation, so the allocator and `alloc`'s
/// growth paths are in both images and the difference is metadata-gen.
fn allocate(input: &str) -> usize {
    let copy: Vec<u8> = input.bytes().collect();
    black_box(copy).len()
}

#[cfg(feature = "metadata-gen")]
fn run(input: &str) -> usize {
    match metadata_gen::extract_metadata(input) {
        Ok(metadata) => {
            metadata.get("title").map_or(0, |title| title.len())
        }
        Err(_) => usize::MAX,
    }
}

#[cfg(not(feature = "metadata-gen"))]
fn run(input: &str) -> usize {
    input.len()
}

/// The entry point the linker keeps; `--gc-sections` drops the rest.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let input = black_box(INPUT);
    black_box(allocate(input));
    black_box(run(input));
    loop {
        core::hint::spin_loop();
    }
}

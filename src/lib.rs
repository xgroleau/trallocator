#![doc = include_str!("../README.md")]
#![no_std]
#![deny(missing_docs)]
#![cfg_attr(feature = "allocator-api", feature(allocator_api))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};

/// A wapper over an allocator that allows tracebility
pub struct Trallocator<A> {
    alloc: A,
    usage: AtomicUsize,
    #[cfg(feature = "max-usage")]
    max_usage: AtomicUsize,
}

impl<A> Trallocator<A> {
    /// Creates a new instance of the traceable allocator using an inner allocator
    pub const fn new(alloc: A) -> Self {
        Self {
            alloc,
            usage: AtomicUsize::new(0),
            #[cfg(feature = "max-usage")]
            max_usage: AtomicUsize::new(0),
        }
    }

    /// Get the number of bytes used in the heap
    pub fn usage(&self) -> usize {
        self.usage.load(Ordering::Relaxed)
    }

    /// Get the max usage of the allocator
    #[cfg(feature = "max-usage")]
    pub fn max_usage(&self) -> usize {
        self.max_usage.load(Ordering::Relaxed)
    }

    /// Clear the max usage and set it to the current usage.
    /// Can be used to track max usage per time frame.
    #[cfg(feature = "max-usage")]
    pub fn clear_max_usage(&self) {
        self.max_usage.store(self.usage(), Ordering::Relaxed);
    }

    /// Borrow the inner allocator.
    /// Note that using the inner allocator will prevent the allocation to be tracked.
    /// Should only be used to init the inner allocator
    pub fn borrow(&self) -> &A {
        &self.alloc
    }

    /// Mutably borrow the inner allocator.
    /// Note that using the inner allocator will prevent the allocation to be tracked.
    /// Should only be used to init the inner allocator
    pub fn borrow_mut(&mut self) -> &mut A {
        &mut self.alloc
    }
}

unsafe impl<A: GlobalAlloc> GlobalAlloc for Trallocator<A> {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let _prev = self.usage.fetch_add(l.size(), Ordering::Acquire);
        // The max usage is the previous value and not the current one
        // but we will have to deallocate eventually anyway and we will get the result there, so we just have a delay until we have the actual maximum
        #[cfg(feature = "max-usage")]
        self.max_usage.fetch_max(_prev, Ordering::Release);
        self.alloc.alloc(l)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, l: Layout) {
        let _prev = self.usage.fetch_sub(l.size(), Ordering::Relaxed);
        #[cfg(feature = "max-usage")]
        self.max_usage.fetch_max(_prev, Ordering::Release);
        self.alloc.dealloc(ptr, l)
    }
}

#[cfg(feature = "allocator-api")]
mod allocator_api {
    use core::alloc::{Allocator, Layout};
    use core::sync::atomic::Ordering;

    use crate::Trallocator;

    unsafe impl<A: Allocator> Allocator for Trallocator<A> {
        fn allocate(
            &self,
            layout: Layout,
        ) -> Result<core::ptr::NonNull<[u8]>, core::alloc::AllocError> {
            let _prev = self.usage.fetch_add(layout.size(), Ordering::Relaxed);
            // The max usage is the previous value and not the current one
            // but we will have to deallocate eventually anyway and we will get the result there, so we just have a delay until we have the actual maximum
            #[cfg(feature = "max-usage")]
            self.max_usage.fetch_max(_prev, Ordering::Relaxed);
            self.alloc.allocate(layout)
        }

        unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, layout: Layout) {
            let _prev = self.usage.fetch_sub(layout.size(), Ordering::Relaxed);
            #[cfg(feature = "max-usage")]
            self.max_usage.fetch_max(_prev, Ordering::Relaxed);
            self.alloc.deallocate(ptr, layout)
        }
    }
}

#[cfg(test)]
mod test {
    extern crate alloc;
    extern crate std;
    use alloc::vec::Vec;
    use std::alloc::System;

    use crate::Trallocator;

    #[test]
    #[cfg(all(feature = "allocator-api", feature = "max-usage"))]
    pub fn trace_alloc() {
        let tralloc: Trallocator<System> = Trallocator::new(System);
        assert_eq!(tralloc.usage(), 0);

        {
            let mut vec: Vec<u8, _> = Vec::new_in(&tralloc);
            vec.reserve_exact(32);
            assert_eq!(tralloc.usage(), 32);
        }
        // Vec dropped here, heap released
        assert_eq!(tralloc.usage(), 0);
        assert_eq!(tralloc.max_usage(), 32);
    }
}

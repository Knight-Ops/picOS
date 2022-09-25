use core::{
    cell::UnsafeCell,
    sync::atomic::{AtomicBool, Ordering},
};
use lock_api::{GuardSend, Mutex, RawMutex};

pub struct NakedMutex<T> {
    inner: UnsafeCell<T>,
}

impl<T> NakedMutex<T> {
    /// Creates a new mutex
    pub const fn new(value: T) -> Self {
        Self {
            inner: UnsafeCell::new(value),
        }
    }
}

impl<T> NakedMutex<T> {
    /// Borrows the data for the duration of the critical section
    pub fn borrow(&self) -> &T {
        unsafe { &*self.inner.get() }
    }
}

unsafe impl<T> Sync for NakedMutex<T> where T: Send {}

pub struct RawSpinlock(AtomicBool);

unsafe impl RawMutex for RawSpinlock {
    const INIT: Self = RawSpinlock(AtomicBool::new(false));

    type GuardMarker = GuardSend;

    fn lock(&self) {
        while !self.try_lock() {}
    }

    fn try_lock(&self) -> bool {
        if !self.0.load(Ordering::Acquire) {
            self.0.store(true, Ordering::Release);
            true
        } else {
            false
        }
    }

    unsafe fn unlock(&self) {
        self.0.store(false, Ordering::Release)
    }
}

pub type Spinlock<T> = lock_api::Mutex<RawSpinlock, T>;
pub type SpinlockGuard<'a, T> = lock_api::MutexGuard<'a, RawSpinlock, T>;

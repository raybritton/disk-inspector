use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct AtomicCounter {
    value: AtomicUsize
}

impl AtomicCounter {
    pub fn new() -> AtomicCounter {
        return AtomicCounter {
           value: AtomicUsize::new(0)
        };
    }
}

impl AtomicCounter {
    pub fn get(&self) -> usize {
        return self.value.load(Ordering::SeqCst);
    }

    pub fn set(&self, new_value: usize) {
        self.value.store(new_value, Ordering::SeqCst);
    }
}
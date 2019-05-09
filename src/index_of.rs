use crate::inspector::{Disk, DiskItem};
use std::sync::Arc;

pub trait IndexOf<T> {
    fn index_of<F>(&self, predicate: F) -> Option<usize> where F: Fn(&T) -> bool;
}

impl IndexOf<Disk> for Vec<Disk> {
    fn index_of<F>(&self, predicate: F) -> Option<usize> where F: Fn(&Disk) -> bool {
        let mut idx = 0_usize;
        for disk in self {
            if predicate(disk) {
                return Some(idx);
            }
            idx += 1;
        }
        return None;
    }
}

impl IndexOf<Arc<DiskItem>> for Vec<Arc<DiskItem>> {
    fn index_of<F>(&self, predicate: F) -> Option<usize> where F: Fn(&Arc<DiskItem>) -> bool {
        let mut idx = 0_usize;
        for disk in self {
            if predicate(disk) {
                return Some(idx);
            }
            idx += 1;
        }
        return None;
    }
}

impl IndexOf<DiskItem> for Vec<DiskItem> {
    fn index_of<F>(&self, predicate: F) -> Option<usize> where F: Fn(&DiskItem) -> bool {
        let mut idx = 0_usize;
        for disk in self {
            if predicate(disk) {
                return Some(idx);
            }
            idx += 1;
        }
        return None;
    }
}
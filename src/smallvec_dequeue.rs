use std::iter::FromIterator;

use smallvec::{Array, SmallVec};

/// Dequeue version of small vec
pub struct SmallVecDequeue<T: Array> {
    /// Offset to the first element
    front: usize,
    /// Offset to where data should be written
    back: usize,
    buf: SmallVec<T>,
}

impl<T: Array> SmallVecDequeue<T> {
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    pub fn len(&self) -> usize {
        count(self.front, self.back, self.capacity())
    }

    pub fn is_empty(&self) -> bool {
        self.front == self.back
    }

    fn grow(&mut self) {
        let old = self.capacity();
        self.buf.reserve(1);
        let new_cap = self.buf.capacity();

        // Buffer is contiguous; do nothing
        if self.front <= self.back {
        }
        // Head is behind tail
        else if self.back < old - self.front {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    self.buf.as_ptr().add(old),
                    self.buf.as_mut_ptr(),
                    self.back,
                )
            }
        } else {
            let new_front = new_cap - (old - self.front);
            unsafe {
                std::ptr::copy_nonoverlapping(
                    self.buf.as_ptr().add(new_front),
                    self.buf.as_mut_ptr().add(self.front),
                    old - self.front,
                )
            };

            self.front = new_front;
            debug_assert!(self.back < self.front);
        }
        debug_assert!(self.back < self.capacity());
        debug_assert!(self.front < self.capacity());
        debug_assert!(self.capacity().count_ones() == 1);
    }

    pub fn push(&mut self, val: T::Item) {
        dbg!(self.len());
        if self.len() == self.capacity() {
            dbg!("Growing");
            self.grow();
        }
        let back = self.back;
        self.back = (back + 1) & self.capacity();

        eprintln!("{}, {}", back, self.buf.len());
        if back < self.buf.len() {
            self.buf[back] = val;
        } else {
            self.push(val)
        }
    }
}

impl<U: Copy, T: Array<Item = U>> SmallVecDequeue<T> {
    /// Removes the first element and returns it
    pub fn pop_front(&mut self) -> Option<T::Item> {
        if self.is_empty() {
            None
        } else {
            let front = self.front;
            self.front = (front + 1) % self.capacity();
            Some(self.buf[front])
        }
    }
}

/// Calculate the number of elements left to be read in the buffer
#[inline]
fn count(front: usize, back: usize, size: usize) -> usize {
    // size is always a power of 2
    (back.wrapping_sub(front)) & (size - 1)
}

impl<T: Array> FromIterator<T::Item> for SmallVecDequeue<T> {
    fn from_iter<U: IntoIterator<Item = T::Item>>(iter: U) -> Self {
        let buf = SmallVec::from_iter(iter);
        Self {
            front: 0,
            back: buf.len(),
            buf,
        }
    }
}

impl<T: Array> Extend<T::Item> for SmallVecDequeue<T> {
    fn extend<U: IntoIterator<Item = T::Item>>(&mut self, iter: U) {
        let iter = iter.into_iter();

        iter.for_each(|val| self.push(val))
    }
}

use std::marker::PhantomData;

use hecs::{Entity, World};

use crate::Child;

/// Iterates children along with Query `Q`. Children who do not satisfy `Q` will be skipped.
/// Count is known in advanced and will not fold iterator.
pub struct ChildrenIter<'a, T> {
    world: &'a World,
    remaining: usize,
    current: Entity,
    marker: PhantomData<T>,
}

impl<'a, T> ChildrenIter<'a, T> {
    pub fn new(world: &'a World, num_children: usize, current: Entity) -> Self {
        Self {
            world,
            remaining: num_children,
            current,
            marker: PhantomData,
        }
    }
}

impl<'a, T> Iterator for ChildrenIter<'a, T>
where
    T: 'static + Send + Sync,
{
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        self.remaining -= 1;

        let current = self.current;
        let data = match self.world.get::<Child<T>>(current) {
            Ok(data) => data,
            Err(_) => return None,
        };

        self.current = data.next;
        Some(current)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.remaining
    }
}

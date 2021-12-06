use std::{collections::VecDeque, marker::PhantomData};

use hecs::Entity;
use hecs_schedule::GenericWorld;
use smallvec::{smallvec, SmallVec};

use crate::{Child, Hierarchy, Parent};

const STACK_SIZE: usize = 64;

/// Iterates children along with Query `Q`. Children who do not satisfy `Q` will be skipped.
/// Count is known in advanced and will not fold iterator.
pub struct ChildrenIter<'a, W, T> {
    world: &'a W,
    remaining: usize,
    current: Option<Entity>,
    marker: PhantomData<T>,
}

impl<'a, W: GenericWorld, T> ChildrenIter<'a, W, T> {
    pub(crate) fn new(world: &'a W, num_children: usize, current: Option<Entity>) -> Self {
        Self {
            world,
            remaining: num_children,
            current,
            marker: PhantomData,
        }
    }
}

impl<'a, W: GenericWorld, T> Iterator for ChildrenIter<'a, W, T>
where
    T: 'static + Send + Sync,
{
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        self.remaining -= 1;

        let current = self.current?;
        let data = match self.world.try_get::<Child<T>>(current) {
            Ok(data) => data,
            Err(_) => return None,
        };

        self.current = Some(data.next);
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

pub struct AncestorIter<'a, W, T> {
    world: &'a W,
    current: Entity,
    marker: PhantomData<T>,
}

impl<'a, W: GenericWorld, T> AncestorIter<'a, W, T> {
    pub(crate) fn new(world: &'a W, current: Entity) -> Self {
        Self {
            world,
            current,
            marker: PhantomData,
        }
    }
}

impl<'a, W: GenericWorld, T: 'static + Send + Sync> Iterator for AncestorIter<'a, W, T> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        self.world
            .try_get::<Child<T>>(self.current)
            .ok()
            .map(|child| {
                self.current = child.parent;
                child.parent
            })
    }
}

#[derive(Debug)]
struct StackFrame {
    current: Entity,
    remaining: usize,
}

pub struct DepthFirstIterator<'a, W, T> {
    world: &'a W,
    marker: PhantomData<T>,
    /// Since StackFrame is so small, use smallvec optimizations
    stack: SmallVec<[StackFrame; STACK_SIZE]>,
}

impl<'a, W: GenericWorld, T: 'static + Send + Sync> DepthFirstIterator<'a, W, T> {
    pub(crate) fn new(world: &'a W, root: Entity) -> Self {
        let stack = world
            .try_get::<Parent<T>>(root)
            .and_then(|parent| {
                let first_child = parent.first_child(world)?;
                Ok(smallvec![StackFrame {
                    current: first_child,
                    remaining: parent.num_children,
                }])
            })
            .unwrap_or_default();

        Self {
            world,
            stack,
            marker: PhantomData,
        }
    }
}

impl<'a, W: GenericWorld, T: 'static + Send + Sync> Iterator for DepthFirstIterator<'a, W, T> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        // The the topmost stackframe
        let top = self.stack.last_mut()?;

        // There are more children in current stackframe
        if top.remaining > 0 {
            let current = top.current;

            let data = self.world.try_get::<Child<T>>(top.current).ok().unwrap();

            // Go to the next child in the linked list of children
            top.current = data.next;
            top.remaining -= 1;

            // If current is a parent, push a new stack frame with the first child
            if let Ok(parent) = self.world.try_get::<Parent<T>>(current) {
                self.stack.push(StackFrame {
                    current: parent.first_child(self.world).unwrap(),
                    remaining: parent.num_children,
                })
            }

            Some(current)
        } else {
            // End of linked list of children, pop stack frame
            self.stack.pop();
            self.next()
        }
    }
}

pub struct BreadthFirstIterator<'a, W, T> {
    world: &'a W,
    marker: PhantomData<T>,
    queue: VecDeque<Entity>,
}

impl<'a, W: GenericWorld + Hierarchy, T: 'static + Send + Sync> BreadthFirstIterator<'a, W, T> {
    pub(crate) fn new(world: &'a W, root: Entity) -> Self {
        // Add immediate children of root to queue
        let queue = world.children::<T>(root).collect();

        Self {
            world,
            queue,
            marker: PhantomData,
        }
    }
}

impl<'a, W: GenericWorld + Hierarchy, T: 'static + Send + Sync> Iterator
    for BreadthFirstIterator<'a, W, T>
{
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        let front = self.queue.pop_front()?;

        // Add any potention children of front to the back of queue
        self.queue.extend(self.world.children::<T>(front));

        Some(front)
    }
}

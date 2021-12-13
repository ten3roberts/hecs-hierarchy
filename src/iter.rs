use std::{collections::VecDeque, marker::PhantomData};

use hecs::{Column, Component, Entity};
use hecs_schedule::GenericWorld;
use smallvec::{smallvec, SmallVec};

use crate::{Child, Hierarchy, Parent};

const STACK_SIZE: usize = 64;

/// Iterates children along with Query `Q`. Children who do not satisfy `Q` will be skipped.
/// Count is known in advanced and will not fold iterator.
pub struct ChildrenIter<'a, T: Component> {
    children: Column<'a, Child<T>>,
    remaining: usize,
    current: Option<Entity>,
    marker: PhantomData<T>,
}

impl<'a, T: Component> ChildrenIter<'a, T> {
    pub(crate) fn new<W: GenericWorld>(
        world: &'a W,
        num_children: usize,
        current: Option<Entity>,
    ) -> Self {
        Self {
            children: world.try_get_column().unwrap(),
            remaining: num_children,
            current,
            marker: PhantomData,
        }
    }
}

impl<'a, T> Iterator for ChildrenIter<'a, T>
where
    T: Component,
{
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        self.remaining -= 1;

        let current = self.current?;
        let data = match self.children.get(current) {
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

pub struct AncestorIter<'a, T: Component> {
    children: Column<'a, Child<T>>,
    current: Entity,
    marker: PhantomData<T>,
}

impl<'a, T: Component> AncestorIter<'a, T> {
    pub(crate) fn new<W: GenericWorld>(world: &'a W, current: Entity) -> Self {
        Self {
            children: world.try_get_column().unwrap(),
            current,
            marker: PhantomData,
        }
    }
}

impl<'a, T: Component> Iterator for AncestorIter<'a, T> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(child) = self.children.get(self.current).ok() {
            self.current = child.parent;
            Some(child.parent)
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct StackFrame {
    current: Entity,
    remaining: usize,
}

pub struct DepthFirstIterator<'a, T: Component> {
    children: Column<'a, Child<T>>,
    parents: Column<'a, Parent<T>>,
    marker: PhantomData<T>,
    /// Since StackFrame is so small, use smallvec optimizations
    stack: SmallVec<[StackFrame; STACK_SIZE]>,
}

impl<'a, T: Component> DepthFirstIterator<'a, T> {
    pub(crate) fn new<W: GenericWorld>(world: &'a W, root: Entity) -> Self {
        let children = world.try_get_column().unwrap();
        let parents = world.try_get_column::<Parent<T>>().unwrap();

        let stack = parents
            .get(root)
            .ok()
            .and_then(|parent| {
                let first_child = parent.first_child(world).ok()?;
                Some(smallvec![StackFrame {
                    current: first_child,
                    remaining: parent.num_children,
                }])
            })
            .unwrap_or_default();

        Self {
            children,
            parents,
            stack,
            marker: PhantomData,
        }
    }
}

impl<'a, T: Component> Iterator for DepthFirstIterator<'a, T> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        // The the topmost stackframe
        let top = self.stack.last_mut()?;

        // There are more children in current stackframe
        if top.remaining > 0 {
            let current = top.current;

            let data = self.children.get(top.current).ok().unwrap();

            // Go to the next child in the linked list of children
            top.current = data.next;
            top.remaining -= 1;

            // If current is a parent, push a new stack frame with the first child
            if let Ok(parent) = self.parents.get(current) {
                self.stack.push(StackFrame {
                    current: parent.column_first_child(&self.children).unwrap(),
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

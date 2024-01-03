use std::{collections::VecDeque, marker::PhantomData};

use hecs::{Component, Entity, QueryBorrow};
use hecs_schedule::GenericWorld;
use smallvec::{smallvec, SmallVec};

use crate::{Child, Hierarchy, Parent};

const STACK_SIZE: usize = 64;

/// Iterates children along with Query `Q`. Children who do not satisfy `Q` will be skipped.
/// Count is known in advanced and will not fold iterator.
pub struct ChildrenIter<'a, T: Component> {
    query: QueryBorrow<'a, &'a Child<T>>,
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
            query: world.try_query().unwrap(),
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
        let view = self.query.view();
        let data = view.get(current)?;

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
    query: QueryBorrow<'a, &'a Child<T>>,
    current: Entity,
    marker: PhantomData<T>,
}

impl<'a, T: Component> AncestorIter<'a, T> {
    pub(crate) fn new<W: GenericWorld>(world: &'a W, current: Entity) -> Self {
        Self {
            query: world.try_query().unwrap(),
            current,
            marker: PhantomData,
        }
    }
}

impl<'a, T: Component> Iterator for AncestorIter<'a, T> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(child) = self.query.view().get(self.current) {
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
    children: QueryBorrow<'a, &'a Child<T>>,
    parents: QueryBorrow<'a, &'a Parent<T>>,
    marker: PhantomData<T>,
    /// Since StackFrame is so small, use smallvec optimizations
    stack: SmallVec<[StackFrame; STACK_SIZE]>,
}

impl<'a, T: Component> DepthFirstIterator<'a, T> {
    pub(crate) fn new<W: GenericWorld>(world: &'a W, root: Entity) -> Self {
        let children = world.try_query().unwrap();
        let mut parents = world.try_query::<&Parent<T>>().unwrap();

        let stack = parents
            .view()
            .get(root)
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

pub struct DepthFirstVisitor<'a, W, T: Component, F> {
    world: &'a W,
    children: QueryBorrow<'a, &'a Child<T>>,
    parents: QueryBorrow<'a, &'a Parent<T>>,
    marker: PhantomData<T>,
    /// Since StackFrame is so small, use smallvec optimizations
    stack: SmallVec<[StackFrame; STACK_SIZE]>,
    accept: F,
}

impl<'a, F: Fn(&W, Entity) -> bool + Component, W: GenericWorld, T: Component>
    DepthFirstVisitor<'a, W, T, F>
{
    pub(crate) fn new(world: &'a W, root: Entity, accept: F) -> Self {
        let children = world.try_query().unwrap();
        let mut parents = world.try_query::<&Parent<T>>().unwrap();

        let stack = parents
            .view()
            .get(root)
            .and_then(|parent| {
                if (accept)(world, root) {
                    let first_child = parent.first_child(world).ok()?;
                    Some(smallvec![StackFrame {
                        current: first_child,
                        remaining: parent.num_children,
                    }])
                } else {
                    None
                }
            })
            .unwrap_or_default();

        Self {
            world,
            accept,
            children,
            parents,
            stack,
            marker: PhantomData,
        }
    }
}

impl<'a, F: Fn(&W, Entity) -> bool + Component, W: GenericWorld, T: Component> Iterator
    for DepthFirstVisitor<'a, W, T, F>
{
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // The the topmost stackframe
            let top = self.stack.last_mut()?;
            // There are more children in current stackframe
            if top.remaining > 0 {
                let current = top.current;

                let children = self.children.view();
                let data = children.get(top.current).unwrap();

                // Go to the next child in the linked list of children
                top.current = data.next;
                top.remaining -= 1;

                if !(self.accept)(self.world, current) {
                    continue;
                }

                // If current is a parent, push a new stack frame with the first child
                if let Some(parent) = self.parents.view().get(current) {
                    self.stack.push(StackFrame {
                        current: parent.view_first_child(&children).unwrap(),
                        remaining: parent.num_children,
                    })
                }

                return Some(current);
            } else {
                // End of linked list of children, pop stack frame
                self.stack.pop();
            }
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

            let children = self.children.view();
            let data = children.get(top.current).unwrap();

            // Go to the next child in the linked list of children
            top.current = data.next;
            top.remaining -= 1;

            // If current is a parent, push a new stack frame with the first child
            if let Some(parent) = self.parents.view().get(current) {
                self.stack.push(StackFrame {
                    current: parent.view_first_child(&children).unwrap(),
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

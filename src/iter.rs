use std::marker::PhantomData;

use hecs::{Entity, World};

use crate::{Child, Parent};

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

pub struct AncestorIter<'a, T> {
    world: &'a World,
    current: Entity,
    marker: PhantomData<T>,
}

impl<'a, T> AncestorIter<'a, T> {
    pub(crate) fn new(world: &'a World, current: Entity) -> Self {
        Self {
            world,
            current,
            marker: PhantomData,
        }
    }
}

impl<'a, T: 'static + Send + Sync> Iterator for AncestorIter<'a, T> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        self.world.get::<Child<T>>(self.current).ok().map(|child| {
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

pub struct DepthFirstIterator<'a, T> {
    world: &'a World,
    marker: PhantomData<T>,
    stack: Vec<StackFrame>,
}

impl<'a, T: 'static + Send + Sync> DepthFirstIterator<'a, T> {
    pub fn new(world: &'a World, root: Entity) -> Self {
        let stack = match world.get::<Parent<T>>(root) {
            Ok(p) => vec![StackFrame {
                current: p.first_child,
                remaining: p.num_children,
            }],
            Err(_) => vec![],
        };

        Self {
            world,
            stack,
            marker: PhantomData,
        }
    }
}

impl<'a, T: 'static + Send + Sync> Iterator for DepthFirstIterator<'a, T> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        let top = self.stack.last_mut()?;

        if top.remaining > 0 {
            let current = top.current;

            let data = self.world.get::<Child<T>>(top.current).ok()?;

            top.current = data.next;
            top.remaining -= 1;

            if let Ok(parent) = self.world.get::<Parent<T>>(current) {
                self.stack.push(StackFrame {
                    current: parent.first_child,
                    remaining: parent.num_children,
                })
            }

            Some(current)
        } else {
            self.stack.pop();
            self.next()
        }
    }
}

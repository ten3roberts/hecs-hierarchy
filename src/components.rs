use std::marker::PhantomData;

use hecs::{Entity, View};
use hecs_schedule::{error::Result, GenericWorld};

/// Component of a entity with descendents in hierarchy tree `T`.
/// Children represent a circular linked list. Since `Parent` and child is generic over a marker
/// type, several hierarchies can coexist.
pub struct Parent<T> {
    pub(crate) num_children: usize,
    pub(crate) last_child: Entity,
    marker: PhantomData<T>,
}

impl<T: 'static + Send + Sync> Parent<T> {
    pub(crate) fn new(num_children: usize, last_child: Entity) -> Self {
        Self {
            num_children,
            last_child,
            marker: PhantomData,
        }
    }

    /// Return the parent's num children.
    pub fn num_children(&self) -> usize {
        self.num_children
    }

    /// Query the parent's first child.
    pub fn first_child<W: GenericWorld>(&self, world: &W) -> Result<Entity> {
        Ok(world.try_get::<Child<T>>(self.last_child)?.next)
    }

    /// Query the parent's first child.
    pub fn view_first_child(&self, view: &View<&Child<T>>) -> Result<Entity> {
        Ok(view
            .get(self.last_child)
            .ok_or_else(|| hecs_schedule::Error::NoSuchEntity(self.last_child))?
            .next)
    }
    /// Return the parent's last child.
    pub fn last_child(&self) -> Entity {
        self.last_child
    }
}

impl<T> std::fmt::Debug for Parent<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Parent")
            .field("num_children", &self.num_children)
            .field("last_child", &self.last_child)
            .finish()
    }
}

/// Component of a child entity in hierarchy tree `T`.
/// Children represent a circular linked list. Since `Parent` and child is generic over a marker
/// type, several hierarchies can coexist.
pub struct Child<T> {
    pub(crate) parent: Entity,
    pub(crate) next: Entity,
    pub(crate) prev: Entity,
    marker: PhantomData<T>,
}

impl<T> Child<T> {
    pub(crate) fn new(parent: Entity, next: Entity, prev: Entity) -> Self {
        Self {
            parent,
            next,
            prev,
            marker: PhantomData,
        }
    }
}

impl<T> std::fmt::Debug for Child<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Child")
            .field("parent", &self.parent)
            .field("next", &self.next)
            .field("prev", &self.prev)
            .finish()
    }
}

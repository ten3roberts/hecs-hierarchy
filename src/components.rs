use std::marker::PhantomData;

use hecs::Entity;

/// Component of a entity with descendents in hierarchy tree `T`.
/// Children represent a circular linked list. Since `Parent` and child is generic over a marker
/// type, several hierarchies can coexist.
pub struct Parent<T> {
    pub(crate) num_children: usize,
    pub(crate) first_child: Entity,
    marker: PhantomData<T>,
}

impl<T> Parent<T> {
    pub fn new(num_children: usize, first_child: Entity) -> Self {
        Self {
            num_children,
            first_child,
            marker: PhantomData,
        }
    }

    /// Return the parent's num children.
    pub fn num_children(&self) -> usize {
        self.num_children
    }

    /// Return the aparent's first child.
    pub fn first_child(&self) -> Entity {
        self.first_child
    }
}

/// Component of a child entity in hierarchy tree `T`.
/// Children represent a circular linked list. Since `Parent` and child is generic over a marker
/// type, several hierarchies can coexist.
pub struct Child<T> {
    pub(crate) next: Entity,
    marker: PhantomData<T>,
}

impl<T> Child<T> {
    pub fn new(next: Entity) -> Self {
        Self {
            next,
            marker: PhantomData,
        }
    }
}

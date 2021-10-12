use std::marker::PhantomData;

use hecs::Entity;

/// Component of a entity with descendents in hierarchy tree `T`.
/// Children represent a circular linked list. Since `Parent` and child is generic over a marker
/// type, several hierarchies can coexist.
#[derive(Debug)]
pub struct Parent<T> {
    pub(crate) num_children: usize,
    pub(crate) last_child: Entity,
    marker: PhantomData<T>,
}

impl<T> Parent<T> {
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

    /// Return the parent's last child.
    pub fn last_child(&self) -> Entity {
        self.last_child
    }
}

/// Component of a child entity in hierarchy tree `T`.
/// Children represent a circular linked list. Since `Parent` and child is generic over a marker
/// type, several hierarchies can coexist.
#[derive(Debug)]
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

// impl<T> std::fmt::Debug for Child<T> {
// fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//     write!(
//         f,
//         "{{ parent: {:?}, next: {:?}, prev: {:?} }}",
//         self.parent, self.next, self.prev
//     )
// }
// }

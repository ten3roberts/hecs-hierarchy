use std::mem;

use hecs::{ComponentError, Entity, World};

use crate::{AncestorIter, Child, ChildrenIter, Parent};

/// A trait for modifying the worlds hierarchy. Implemented for `hecs::World`>
pub trait Hierarchy<E> {
    /// Attach `child` to `parent`. Parent does not require an existing `Parent component`. Returns
    /// the passed child. The child is inserted at the head of the list.
    fn attach<T: 'static + Send + Sync>(
        &mut self,
        child: Entity,
        parent: Entity,
    ) -> Result<Entity, E>;

    /// Traverses the immediate children of parent. If parent is not a Parent, an empty iterator is
    /// returned.
    fn children<T: 'static + Send + Sync>(&self, parent: Entity) -> ChildrenIter<T>;
    /// Traverse the tree upwards. Iterator does not include the child itself.
    fn ancestors<T: 'static + Send + Sync>(&self, child: Entity) -> AncestorIter<T>;
}

impl Hierarchy<ComponentError> for World {
    fn attach<T: 'static + Send + Sync>(
        &mut self,
        child: Entity,
        parent: Entity,
    ) -> Result<Entity, ComponentError> {
        let mut maybe_p = self.get_mut::<Parent<T>>(parent);
        if let Ok(ref mut p) = maybe_p {
            p.num_children += 1;
            let next = p.first_child;
            p.first_child = child;

            let mut next_data = self.get_mut::<Child<T>>(next)?;
            let prev = next_data.prev;
            next_data.prev = child;

            mem::drop(next_data);
            mem::drop(maybe_p);

            // Update backward linking
            {
                let mut prev_data = self.get_mut::<Child<T>>(prev)?;
                prev_data.next = child;
            }

            self.insert_one(child, Child::<T>::new(parent, next, prev))?;

            return Ok(child);
        }

        mem::drop(maybe_p);

        // Parent component didn't exist
        self.insert_one(parent, Parent::<T>::new(1, child))?;

        self.insert_one(child, Child::<T>::new(parent, child, child))?;

        Ok(child)
    }

    fn children<T: 'static + Send + Sync>(&self, parent: Entity) -> ChildrenIter<T> {
        match self.get::<Parent<T>>(parent) {
            Ok(p) => ChildrenIter::new(self, p.num_children, p.first_child),
            // Return an iterator that does nothing.
            Err(_) => ChildrenIter::new(self, 0, Entity::from_bits(0)),
        }
    }

    fn ancestors<T: 'static + Send + Sync>(&self, child: Entity) -> AncestorIter<T> {
        AncestorIter::new(self, child)
    }
}

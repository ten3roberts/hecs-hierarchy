use std::mem;

use hecs::{ComponentError, DynamicBundle, Entity, World};

use crate::{AncestorIter, BreadthFirstIterator, Child, ChildrenIter, DepthFirstIterator, Parent};

/// A trait for modifying the worlds hierarchy. Implemented for `hecs::World`>
pub trait Hierarchy<E> {
    /// Attach `child` to `parent`. Parent does not require an existing `Parent component`. Returns
    /// the passed child. The child is inserted at the head of the list.
    /// *Note*: The entity needs to be explicitely detached before being removed.
    fn attach<T: 'static + Send + Sync>(
        &mut self,
        child: Entity,
        parent: Entity,
    ) -> Result<Entity, E>;

    /// Attach a new entity with specified components to `parent`. Parent does not require an existing `Parent component`. Returns
    /// the passed child. The child is inserted at the head of the list.
    fn attach_new<T: 'static + Send + Sync, C: DynamicBundle>(
        &mut self,
        parent: Entity,
        components: C,
    ) -> Result<Entity, E>;

    /// Detaches all children from entity and detaches entity from parent. Use this before removing
    /// entities to ensure no loose entity ids.
    fn detach_all<T: 'static + Send + Sync>(
        &mut self,
        entity: Entity,
    ) -> Result<(), ComponentError>;

    /// Detaches all children of parent.
    fn detach_children<T: 'static + Send + Sync>(
        &mut self,
        parent: Entity,
    ) -> Result<(), ComponentError>;

    /// Returns the parent entity of child.
    fn parent<T: 'static + Send + Sync>(&self, child: Entity) -> Result<Entity, ComponentError>;

    /// Detach the child from tree `T`. The children of `child` will not remain in hierachy, but will
    /// remain attached to `child`, which means a later attach also will attach the children of `child`
    /// into the hierarchy. Essentially moving the subtree.
    fn detach<T: 'static + Send + Sync>(&mut self, child: Entity) -> Result<(), ComponentError>;

    /// Despawn parent and all children recursively. Essentially despawns a whole subtree including
    /// root. Does not fail if there are invalid, dangling IDs in tree.
    fn despawn_all<T: 'static + Send + Sync>(&mut self, parent: Entity);

    /// Traverses the immediate children of parent. If parent is not a Parent, an empty iterator is
    /// returned.
    fn children<T: 'static + Send + Sync>(&self, parent: Entity) -> ChildrenIter<T>;

    /// Traverse the tree upwards. Iterator does not include the child itself.
    fn ancestors<T: 'static + Send + Sync>(&self, child: Entity) -> AncestorIter<T>;

    /// Traverse the tree depth first. Iterator does not include the child itself.
    fn descendants_depth_first<T: 'static + Send + Sync>(
        &self,
        root: Entity,
    ) -> DepthFirstIterator<T>;

    /// Traverse the tree breadth first. Iterator does not include the child itself.
    fn descendants_breadth_first<T: 'static + Send + Sync>(
        &self,
        root: Entity,
    ) -> BreadthFirstIterator<T>;
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

    fn detach_all<T: 'static + Send + Sync>(
        &mut self,
        entity: Entity,
    ) -> Result<(), ComponentError> {
        self.detach_children::<T>(entity)?;
        self.detach::<T>(entity)?;
        Ok(())
    }

    /// Detaches all children of parent.
    fn detach_children<T: 'static + Send + Sync>(
        &mut self,
        parent: Entity,
    ) -> Result<(), ComponentError> {
        let children = self.children::<T>(parent).collect::<Vec<Entity>>();

        children
            .iter()
            .try_for_each(|child| self.remove_one::<Child<T>>(*child).map(|_| ()))?;

        Ok(())
    }

    fn parent<T: 'static + Send + Sync>(&self, child: Entity) -> Result<Entity, ComponentError> {
        self.get::<Child<T>>(child).map(|child| child.parent)
    }

    fn detach<T: 'static + Send + Sync>(&mut self, child: Entity) -> Result<(), ComponentError> {
        let data = self.get_mut::<Child<T>>(child)?;
        let parent = data.parent;
        let prev = data.prev;
        let next = data.next;

        mem::drop(data);

        self.get_mut::<Child<T>>(prev)?.next = next;
        self.get_mut::<Child<T>>(next)?.prev = prev;
        self.get_mut::<Parent<T>>(parent)?.num_children -= 1;

        Ok(())
    }

    fn despawn_all<T: 'static + Send + Sync>(&mut self, parent: Entity) {
        let to_despawn = self
            .descendants_depth_first::<T>(parent)
            .collect::<Vec<_>>();

        // Detach from parent if necessary
        let _ = self.detach::<T>(parent);

        // Should not panic since we just
        to_despawn.iter().for_each(|entity| {
            let _ = self.despawn(*entity);
        });

        let _ = self.despawn(parent);
    }

    fn attach_new<T: 'static + Send + Sync, C: DynamicBundle>(
        &mut self,
        parent: Entity,
        components: C,
    ) -> Result<Entity, ComponentError> {
        let child = self.spawn(components);
        self.attach::<T>(child, parent)
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

    fn descendants_depth_first<T: 'static + Send + Sync>(
        &self,
        root: Entity,
    ) -> DepthFirstIterator<T> {
        DepthFirstIterator::new(self, root)
    }

    /// Traverse the tree breadth first. Iterator does not include the child itself.
    fn descendants_breadth_first<T: 'static + Send + Sync>(
        &self,
        root: Entity,
    ) -> BreadthFirstIterator<T> {
        BreadthFirstIterator::new(self, root)
    }
}

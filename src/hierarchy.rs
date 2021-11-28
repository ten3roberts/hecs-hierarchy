use std::mem;

use hecs::{ComponentError, DynamicBundle, Entity, QueryBorrow, Without, World};

use crate::{AncestorIter, BreadthFirstIterator, Child, ChildrenIter, DepthFirstIterator, Parent};

/// A trait for modifying the worlds hierarchy. Implemented for `hecs::World`>
pub trait Hierarchy {
    /// Attach `child` to `parent`. Parent does not require an existing `Parent component`. Returns
    /// the passed child.
    /// *Note*: The entity needs to be explicitly detached before being removed.
    fn attach<T: 'static + Send + Sync>(
        &mut self,
        child: Entity,
        parent: Entity,
    ) -> Result<Entity, ComponentError>;

    /// Attach a new entity with specified components to `parent`. Parent does not require an existing `Parent component`. Returns
    /// the passed child.
    fn attach_new<T: 'static + Send + Sync, C: DynamicBundle>(
        &mut self,
        parent: Entity,
        components: C,
    ) -> Result<Entity, ComponentError>;

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

    /// Returns an iterator over all root objects in the world
    fn roots<T: 'static + Send + Sync>(&self) -> QueryBorrow<Without<Child<T>, &Parent<T>>>;
}

impl Hierarchy for World {
    fn attach<T: 'static + Send + Sync>(
        &mut self,
        child: Entity,
        parent: Entity,
    ) -> Result<Entity, ComponentError> {
        let mut maybe_p = self.get_mut::<Parent<T>>(parent);
        if let Ok(ref mut p) = maybe_p {
            p.num_children += 1;
            let prev = p.last_child;
            p.last_child = child;

            let mut prev_data = self.get_mut::<Child<T>>(prev)?;
            let next = prev_data.next;
            prev_data.next = child;

            mem::drop(prev_data);
            mem::drop(maybe_p);

            // Update backward linking
            {
                let mut next_data = self.get_mut::<Child<T>>(next)?;
                next_data.prev = child;
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

        let mut parent = self.get_mut::<Parent<T>>(parent)?;
        parent.num_children -= 1;
        if parent.last_child == child {
            parent.last_child = prev;
        }

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
        self.get::<Parent<T>>(parent)
            .and_then(|parent| {
                let first_child = parent.first_child(self)?;

                Ok(ChildrenIter::new(
                    self,
                    parent.num_children,
                    Some(first_child),
                ))
            })
            .unwrap_or_else(move |_| {
                // Return an iterator that does nothing.
                ChildrenIter::new(self, 0, None)
            })
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

    fn roots<T: 'static + Send + Sync>(&self) -> QueryBorrow<Without<Child<T>, &Parent<T>>> {
        self.query::<&Parent<T>>().without::<Child<T>>()
    }
}

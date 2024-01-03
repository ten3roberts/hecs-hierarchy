use std::mem;

use hecs::{Component, DynamicBundle, Entity, QueryBorrow, Without, World};
use hecs_schedule::{error::Result, GenericWorld};

use crate::{
    AncestorIter, BreadthFirstIterator, Child, ChildrenIter, DepthFirstIterator, DepthFirstVisitor,
    Parent,
};

/// A trait for modifying the worlds hierarchy. Implemented for `hecs::World`>
pub trait HierarchyMut {
    /// Attach `child` to `parent`. Parent does not require an existing `Parent component`. Returns
    /// the passed child.
    /// *Note*: The entity needs to be explicitly detached before being removed.
    fn attach<T: Component>(&mut self, child: Entity, parent: Entity) -> Result<Entity>;

    /// Attach a new entity with specified components to `parent`. Parent does not require an existing `Parent component`. Returns
    /// the passed child.
    fn attach_new<T: Component, C: DynamicBundle>(
        &mut self,
        parent: Entity,
        components: C,
    ) -> Result<Entity>;

    /// Detaches all children from entity and detaches entity from parent. Use this before removing
    /// entities to ensure no loose entity ids.
    fn detach_all<T: Component>(&mut self, entity: Entity) -> Result<()>;

    /// Detaches all children of parent.
    fn detach_children<T: Component>(&mut self, parent: Entity) -> Result<Vec<Entity>>;
    fn despawn_children<T: Component>(&mut self, parent: Entity) -> Result<()>;

    /// Detach the child from tree `T`. The children of `child` will not remain in hierachy, but will
    /// remain attached to `child`, which means a later attach also will attach the children of `child`
    /// into the hierarchy. Essentially moving the subtree.
    fn detach<T: Component>(&mut self, child: Entity) -> Result<()>;

    /// Despawn parent and all children recursively. Essentially despawns a whole subtree including
    /// root. Does not fail if there are invalid, dangling IDs in tree.
    fn despawn_all<T: Component>(&mut self, parent: Entity);
}

/// Non mutating part of hierarchy
pub trait Hierarchy
where
    Self: Sized,
{
    /// Returns the parent entity of child.
    fn parent<T: Component>(&self, child: Entity) -> Result<Entity>;

    fn root<T: Component>(&self, child: Entity) -> Result<Entity>;

    /// Traverses the immediate children of parent. If parent is not a Parent, an empty iterator is
    /// returned.
    fn children<T: Component>(&self, parent: Entity) -> ChildrenIter<T>;

    /// Traverse the tree upwards. Iterator does not include the child itself.
    fn ancestors<T: Component>(&self, child: Entity) -> AncestorIter<T>;

    /// Traverse the tree depth first. Iterator does not include the child itself.
    fn descendants_depth_first<T: Component>(&self, root: Entity) -> DepthFirstIterator<T>;

    /// Traverse the tree depth first with an acceptance function
    fn visit<T: Component, F: Fn(&Self, Entity) -> bool + Component>(
        &self,
        root: Entity,
        accept: F,
    ) -> DepthFirstVisitor<Self, T, F>;

    /// Traverse the tree breadth first. Iterator does not include the child itself.
    fn descendants_breadth_first<T: Component>(
        &self,
        root: Entity,
    ) -> BreadthFirstIterator<Self, T>;

    /// Returns an iterator over all root objects in the world
    fn roots<T: Component>(&self) -> Result<QueryBorrow<Without<&Parent<T>, &Child<T>>>>;
}

impl HierarchyMut for World {
    fn attach<T: Component>(&mut self, child: Entity, parent: Entity) -> Result<Entity> {
        let mut maybe_p = self.try_get_mut::<Parent<T>>(parent);
        if let Ok(ref mut p) = maybe_p {
            p.num_children += 1;
            let prev = p.last_child;
            p.last_child = child;

            let mut prev_data = self.try_get_mut::<Child<T>>(prev)?;
            let next = prev_data.next;
            prev_data.next = child;

            mem::drop(prev_data);
            mem::drop(maybe_p);

            // Update backward linking
            {
                let mut next_data = self.try_get_mut::<Child<T>>(next)?;
                next_data.prev = child;
            }

            self.try_insert(child, (Child::<T>::new(parent, next, prev),))?;

            return Ok(child);
        }

        mem::drop(maybe_p);

        // Parent component didn't exist
        self.try_insert(parent, (Parent::<T>::new(1, child),))?;

        self.try_insert(child, (Child::<T>::new(parent, child, child),))?;

        Ok(child)
    }

    fn attach_new<T: Component, C: DynamicBundle>(
        &mut self,
        parent: Entity,
        components: C,
    ) -> Result<Entity> {
        let child = self.spawn(components);
        self.attach::<T>(child, parent)
    }

    fn detach_all<T: Component>(&mut self, entity: Entity) -> Result<()> {
        self.detach_children::<T>(entity)?;
        self.detach::<T>(entity)?;
        Ok(())
    }

    /// Detaches all children of parent.
    fn detach_children<T: Component>(&mut self, parent: Entity) -> Result<Vec<Entity>> {
        let children = self.children::<T>(parent).collect::<Vec<Entity>>();

        children.iter().try_for_each(|child| -> Result<_> {
            self.try_remove_one::<Child<T>>(*child)?;
            Ok(())
        })?;

        self.remove_one::<Parent<T>>(parent).unwrap();

        Ok(children)
    }

    /// Detaches all children of parent.
    fn despawn_children<T: Component>(&mut self, parent: Entity) -> Result<()> {
        let children = self.children::<T>(parent).collect::<Vec<Entity>>();

        children
            .iter()
            .for_each(|child| self.despawn_all::<Child<T>>(*child));

        self.remove_one::<Parent<T>>(parent).unwrap();

        Ok(())
    }

    fn detach<T: Component>(&mut self, child: Entity) -> Result<()> {
        let data = self.try_get_mut::<Child<T>>(child)?;
        let parent = data.parent;
        let prev = data.prev;
        let next = data.next;

        mem::drop(data);

        self.try_get_mut::<Child<T>>(prev)?.next = next;
        self.try_get_mut::<Child<T>>(next)?.prev = prev;

        let mut parent = self.try_get_mut::<Parent<T>>(parent)?;
        parent.num_children -= 1;
        if parent.last_child == child {
            parent.last_child = prev;
        }

        Ok(())
    }

    fn despawn_all<T: Component>(&mut self, parent: Entity) {
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
}

impl<W: GenericWorld> Hierarchy for W {
    fn parent<T: Component>(&self, child: Entity) -> Result<Entity> {
        self.try_get::<Child<T>>(child).map(|child| child.parent)
    }

    fn root<T: Component>(&self, child: Entity) -> Result<Entity> {
        let mut cur = child;
        loop {
            match self.parent::<T>(cur) {
                Ok(val) => cur = val,
                Err(hecs_schedule::Error::MissingComponent(_, _)) => break,
                Err(val) => return Err(val),
            }
        }

        Ok(cur)
    }

    fn children<T: Component>(&self, parent: Entity) -> ChildrenIter<T> {
        self.try_get::<Parent<T>>(parent)
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

    fn ancestors<T: Component>(&self, child: Entity) -> AncestorIter<T> {
        AncestorIter::new(self, child)
    }

    fn descendants_depth_first<T: Component>(&self, root: Entity) -> DepthFirstIterator<T> {
        DepthFirstIterator::new(self, root)
    }

    /// Traverse the tree breadth first. Iterator does not include the child itself.
    fn descendants_breadth_first<T: Component>(
        &self,
        root: Entity,
    ) -> BreadthFirstIterator<Self, T> {
        BreadthFirstIterator::new(self, root)
    }

    fn visit<T: Component, F: Fn(&Self, Entity) -> bool + Component>(
        &self,
        root: Entity,
        accept: F,
    ) -> DepthFirstVisitor<Self, T, F> {
        DepthFirstVisitor::new(self, root, accept)
    }

    fn roots<T: Component>(&self) -> Result<QueryBorrow<Without<&Parent<T>, &Child<T>>>> {
        Ok(self.try_query::<&Parent<T>>()?.without::<&Child<T>>())
    }
}

trait WorldExt {
    fn try_insert(&mut self, e: Entity, c: impl DynamicBundle) -> Result<()>;
    fn try_remove_one<C: Component>(&mut self, e: Entity) -> Result<C>;
}

impl WorldExt for World {
    fn try_insert(&mut self, e: Entity, c: impl DynamicBundle) -> Result<()> {
        self.insert(e, c)
            .map_err(|_| hecs_schedule::Error::NoSuchEntity(e))
    }

    fn try_remove_one<C: Component>(&mut self, e: Entity) -> Result<C> {
        self.remove_one::<C>(e)
            .map_err(|_| hecs_schedule::Error::NoSuchEntity(e))
    }
}

/// A query for defininig a compatible subworld for [Hierarchy]
pub type HierarchyQuery<'a, T> = (&'a Parent<T>, &'a Child<T>);

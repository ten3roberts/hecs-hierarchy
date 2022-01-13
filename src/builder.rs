use std::{
    cell::{Ref, RefCell, RefMut},
    marker::PhantomData,
};

use hecs::{Component, DynamicBundle, Entity, World};

use crate::HierarchyMut;

/// A wrapper to help spawn trees more ergonomically
///
/// # Example
/// ```
/// # use hecs::*;
/// # use hecs_hierarchy::*;
/// #
/// struct TreeMarker;
///
/// let mut world = World::new();
/// let builder = TreeBuilder::<TreeMarker>::new(&mut world);
/// let tree_root = builder
///     .spawn_tree(("root",))
///     .attach_new(("child 1",))
///     .attach({
///         builder
///             .spawn_tree(("child 2",))
///             .attach_new(("child 2.1",))
///             .attach_new(("child 2.2",))
///             .entity()
///     })
///     .entity();
/// ```
pub struct TreeBuilder<'a, T> {
    // World uses interior mutability to allows recursive tree building using
    // the same builder.
    world: RefCell<&'a mut World>,
    marker: PhantomData<T>,
}

impl<'a, T: Component> TreeBuilder<'a, T> {
    pub fn new(world: &'a mut World) -> Self {
        Self {
            world: RefCell::new(world),
            marker: PhantomData,
        }
    }

    /// Immutably borrows the wrapped `&mut World` from the contained `RefCell`.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently mutably borrowed.
    pub(crate) fn world(&self) -> Ref<&'a mut World> {
        self.world.borrow()
    }

    /// Mutably borrows the wrapped `&mut World` from the contained `RefCell`.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed.
    pub(crate) fn world_mut(&self) -> RefMut<&'a mut World> {
        self.world.borrow_mut()
    }

    /// Spawns an entity in the world wrapped in this `TreeBuilder`, without
    /// actually building a tree or attaching it to anything.
    pub fn spawn<C: DynamicBundle>(&self, components: C) -> Entity {
        self.world.borrow_mut().spawn(components)
    }

    /// Spawns a tree root in the world wrapped in this `TreeBuilder`, returning
    /// a [`TreeBuilderAt`] that allows spawning children under it.
    pub fn spawn_tree<C: DynamicBundle>(&self, components: C) -> TreeBuilderAt<'a, '_, T> {
        let root = self.spawn(components);
        TreeBuilderAt {
            builder: self,
            parent: root,
        }
    }
}

/// A wrapper to help spawn children for a given entity more ergonomically
///
/// Created by [`TreeBuilder::spawn_tree`]
pub struct TreeBuilderAt<'a, 'b, T> {
    builder: &'b TreeBuilder<'a, T>,
    parent: Entity,
}

impl<'a, T: 'static + Send + Sync> TreeBuilderAt<'a, '_, T> {
    /// Immutably borrows the wrapped `&mut World` from the contained `RefCell`.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently mutably borrowed.
    pub fn world(&self) -> Ref<&'a mut World> {
        self.builder.world()
    }

    /// Mutably borrows the wrapped `&mut World` from the contained `RefCell`.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed.
    pub fn world_mut(&self) -> RefMut<&'a mut World> {
        self.builder.world_mut()
    }

    /// Spawns an entity, without actually attaching it to anything.
    pub fn spawn<C: DynamicBundle>(&self, components: C) -> Entity {
        self.builder.world.borrow_mut().spawn(components)
    }

    /// Spawns a child under the currenty entity, returning the `Entity`. If the
    /// `Entity` isn't required, you can use `attach_new` instead, which supports
    /// method chaining.
    pub fn spawn_child<C: DynamicBundle>(&self, components: C) -> Entity {
        self.builder
            .world
            .borrow_mut()
            .attach_new::<T, C>(self.parent, components)
            .unwrap()
    }

    /// Attaches a child to the current entity.
    pub fn attach(&self, entity: Entity) -> &Self {
        self.builder
            .world
            .borrow_mut()
            .attach::<T>(entity, self.parent)
            .unwrap();
        self
    }

    /// Spawns a new child entity under the current entity.
    pub fn attach_new<C: DynamicBundle>(&self, components: C) -> &Self {
        self.spawn_child(components);
        self
    }

    pub fn entity(&self) -> Entity {
        self.parent
    }
}

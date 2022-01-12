use std::{
    cell::{Ref, RefCell, RefMut},
    marker::PhantomData,
};

use hecs::{Component, DynamicBundle, DynamicBundleClone, Entity, EntityBuilderClone, World};
use hecs_schedule::{CommandBuffer, GenericWorld};

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

/// Ergonomically construct trees without knowledge of world.
///
/// This struct builds the world using [EntityBuilderClone](hecs::EntityBuilderClone)
///
/// # Example
/// ```rust
/// use hecs_hierarchy::*;
/// use hecs::*;
///
/// struct Tree;
/// let mut world = World::default();
/// let mut builder = DeferredTreeBuilder::<Tree>::from_bundle(("root",));
/// builder.attach_bundle(("child 1",));
/// builder.attach({
///     let mut builder = DeferredTreeBuilder::new();
///     builder.add("child 2");
///     builder
/// });

/// let root = builder.build(&mut world);

/// assert_eq!(*world.get::<&'static str>(root).unwrap(), "root");

/// for (a, b) in world
///     .descendants_depth_first::<Tree>(root)
///     .zip(["child 1", "child 2"])
/// {
///     assert_eq!(*world.get::<&str>(a).unwrap(), b)
/// }
///
/// ```
#[derive(Clone)]
pub struct DeferredTreeBuilder<T> {
    children: Vec<DeferredTreeBuilder<T>>,
    builder: EntityBuilderClone,
    marker: PhantomData<T>,
}

impl<T: Component> DeferredTreeBuilder<T> {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            builder: EntityBuilderClone::new(),
            marker: PhantomData,
        }
    }

    pub fn from_builder(builder: EntityBuilderClone) -> Self {
        Self {
            children: Vec::new(),
            builder,
            marker: PhantomData,
        }
    }

    pub fn from_bundle(bundle: impl DynamicBundleClone) -> Self {
        let mut builder = EntityBuilderClone::new();
        builder.add_bundle(bundle);

        Self {
            children: Vec::new(),
            builder,
            marker: PhantomData,
        }
    }

    pub fn build(&self, world: &mut World) -> Entity {
        let builder = self.builder.clone().build();
        let parent = world.spawn(&builder);

        for child in &self.children {
            let child = child.build(world);
            world.attach::<T>(child, parent).unwrap();
        }

        parent
    }

    pub fn build_cmd(&self, world: &impl GenericWorld, cmd: &mut CommandBuffer) -> Entity {
        let builder = self.builder.clone().build();
        let parent = world.reserve();
        cmd.insert(parent, &builder);

        for child in &self.children {
            let child = child.build_cmd(world, cmd);
            cmd.write(move |w: &mut World| {
                w.attach::<T>(child, parent).unwrap();
            });
        }
        parent
    }

    /// Add a component to the root
    pub fn add(&mut self, component: impl Component + Clone) -> &mut Self {
        self.builder.add(component);
        self
    }

    /// Add a bundle to the root
    pub fn add_bundle(&mut self, bundle: impl DynamicBundleClone) -> &mut Self {
        self.builder.add_bundle(bundle);
        self
    }

    /// Atttach a new subtree
    pub fn attach(&mut self, child: DeferredTreeBuilder<T>) -> &mut Self {
        self.children.push(child);
        self
    }

    /// Attach a new leaf as an entity builder
    pub fn attach_new(&mut self, child: EntityBuilderClone) -> &mut Self {
        self.children.push(Self::from_builder(child));
        self
    }

    /// Attach a new leaf as a bundle
    pub fn attach_bundle(&mut self, child: impl DynamicBundleClone) -> &mut Self {
        self.children.push(Self::from_bundle(child));
        self
    }

    /// Get a reference to the deferred tree builder's builder.
    pub fn builder(&self) -> &EntityBuilderClone {
        &self.builder
    }

    /// Get a reference to the deferred tree builder's children.
    pub fn children(&self) -> &[DeferredTreeBuilder<T>] {
        self.children.as_ref()
    }

    /// Get a mutable reference to the deferred tree builder's builder.
    pub fn builder_mut(&mut self) -> &mut EntityBuilderClone {
        &mut self.builder
    }
}

impl<T: Component> From<EntityBuilderClone> for DeferredTreeBuilder<T> {
    fn from(builder: EntityBuilderClone) -> Self {
        Self::from_builder(builder)
    }
}

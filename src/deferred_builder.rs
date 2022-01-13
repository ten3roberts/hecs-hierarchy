use std::marker::PhantomData;

use hecs::{Component, DynamicBundleClone, Entity, EntityBuilderClone, World};
use hecs_schedule::{CommandBuffer, GenericWorld};

use crate::HierarchyMut;

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

/// let root = builder.spawn(&mut world);

/// assert_eq!(*world.get::<&'static str>(root).unwrap(), "root");

/// for (a, b) in world
///     .descendants_depth_first::<Tree>(root)
///     .zip(["child 1", "child 2"])
/// {
///     assert_eq!(*world.get::<&str>(a).unwrap(), b)
/// }
///
/// ```
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

    pub fn spawn(self, world: &mut World) -> Entity {
        let builder = self.builder.build();
        let parent = world.spawn(&builder);

        for child in self.children {
            let child = child.spawn(world);
            world.attach::<T>(child, parent).unwrap();
        }

        parent
    }

    pub fn spawn_deferred(self, world: &impl GenericWorld, cmd: &mut CommandBuffer) -> Entity {
        let builder = self.builder.build();
        let parent = world.reserve();
        cmd.insert(parent, &builder);

        for child in self.children {
            let child = child.spawn_deferred(world, cmd);
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

impl<T> Clone for DeferredTreeBuilder<T> {
    fn clone(&self) -> Self {
        Self {
            children: self.children.clone(),
            builder: self.builder.clone(),
            marker: PhantomData,
        }
    }
}

impl<B: DynamicBundleClone, T: Component> From<B> for DeferredTreeBuilder<T> {
    fn from(bundle: B) -> Self {
        Self::from_bundle(bundle)
    }
}

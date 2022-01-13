use std::marker::PhantomData;

use hecs::{Component, DynamicBundleClone, Entity, EntityBuilderClone, World};
use hecs_schedule::{CommandBuffer, GenericWorld};

use crate::HierarchyMut;

/// Cloneable version of the [crate::TreeBuilder]
pub struct TreeBuilderClone<T> {
    children: Vec<TreeBuilderClone<T>>,
    builder: EntityBuilderClone,
    marker: PhantomData<T>,
}

impl<T: Component> TreeBuilderClone<T> {
    /// Construct a new empty tree
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            builder: EntityBuilderClone::new(),
            marker: PhantomData,
        }
    }

    /// Spawn the whole tree into the world
    pub fn spawn(self, world: &mut World) -> Entity {
        let builder = self.builder.build();
        let parent = world.spawn(&builder);

        for child in self.children {
            let child = child.spawn(world);
            world.attach::<T>(child, parent).unwrap();
        }

        parent
    }

    /// Spawn the whole tree into a commandbuffer.
    /// The world is required for reserving entities.
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
    pub fn attach_tree(&mut self, child: Self) -> &mut Self {
        self.children.push(child);
        self
    }

    /// Attach a new leaf
    pub fn attach(&mut self, child: impl Into<Self>) -> &mut Self {
        self.children.push(child.into());
        self
    }

    /// Consuming variant of [Self::attach].
    ///
    /// This is useful for nesting to alleviate the need to save an intermediate
    /// builder
    pub fn attach_move(mut self, child: impl Into<Self>) -> Self {
        self.children.push(child.into());
        self
    }

    /// Consuming variant of [Self::attach_tree].
    /// This is useful for nesting to alleviate the need to save an intermediate
    /// builder
    pub fn attach_tree_move(mut self, child: impl Into<Self>) -> Self {
        self.children.push(child.into());
        self
    }

    /// Get a reference to the deferred tree builder's children.
    pub fn children(&self) -> &[Self] {
        self.children.as_ref()
    }

    /// Get a reference to the deferred tree builder's root.
    pub fn root(&self) -> &EntityBuilderClone {
        &self.builder
    }

    /// Get a mutable reference to the deferred tree builder's builder.
    pub fn root_mut(&mut self) -> &mut EntityBuilderClone {
        &mut self.builder
    }
}

impl<T> Clone for TreeBuilderClone<T> {
    fn clone(&self) -> Self {
        Self {
            children: self.children.clone(),
            builder: self.builder.clone(),
            marker: PhantomData,
        }
    }
}

impl<B: DynamicBundleClone, T: Component> From<B> for TreeBuilderClone<T> {
    fn from(bundle: B) -> Self {
        let mut builder = EntityBuilderClone::new();
        builder.add_bundle(bundle);

        Self {
            children: Vec::new(),
            builder,
            marker: PhantomData,
        }
    }
}

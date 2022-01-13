use std::marker::PhantomData;

use hecs::{Component, DynamicBundle, Entity, EntityBuilder, World};
use hecs_schedule::{CommandBuffer, GenericWorld};

use crate::{HierarchyMut, TreeBuilderClone};

/// Ergonomically construct trees without knowledge of world.
///
/// This struct builds the world using [EntityBuilder](hecs::EntityBuilder)
///
/// # Example
/// ```rust
/// use hecs_hierarchy::*;
/// use hecs::*;
///
/// struct Tree;
/// let mut world = World::default();
/// let mut builder = TreeBuilder::<Tree>::from(("root",));
/// builder.attach(("child 1",));
/// builder.attach({
///     let mut builder = TreeBuilder::new();
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
pub struct TreeBuilder<T> {
    children: Vec<TreeBuilder<T>>,
    builder: EntityBuilder,
    marker: PhantomData<T>,
}

impl<T: Component> TreeBuilder<T> {
    /// Construct a new empty tree
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            builder: EntityBuilder::new(),
            marker: PhantomData,
        }
    }

    /// Spawn the whole tree into the world
    pub fn spawn(&mut self, world: &mut World) -> Entity {
        let builder = self.builder.build();
        let parent = world.spawn(builder);

        for mut child in self.children.drain(..) {
            let child = child.spawn(world);
            world.attach::<T>(child, parent).unwrap();
        }

        parent
    }

    /// Spawn the whole tree into a commandbuffer.
    /// The world is required for reserving entities.
    pub fn spawn_deferred(&mut self, world: &impl GenericWorld, cmd: &mut CommandBuffer) -> Entity {
        let builder = self.builder.build();
        let parent = world.reserve();
        cmd.insert(parent, builder);

        for mut child in self.children.drain(..) {
            let child = child.spawn_deferred(world, cmd);
            cmd.write(move |w: &mut World| {
                w.attach::<T>(child, parent).unwrap();
            });
        }
        parent
    }

    /// Add a component to the root
    pub fn add(&mut self, component: impl Component) -> &mut Self {
        self.builder.add(component);
        self
    }

    /// Add a bundle to the root
    pub fn add_bundle(&mut self, bundle: impl DynamicBundle) -> &mut Self {
        self.builder.add_bundle(bundle);
        self
    }

    /// Atttach a new subtree
    pub fn attach_tree(&mut self, child: Self) -> &mut Self {
        self.children.push(child);
        self
    }

    /// Attach a new leaf as a bundle
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
    pub fn root(&self) -> &EntityBuilder {
        &self.builder
    }

    /// Get a mutable reference to the deferred tree builder's root.
    pub fn root_mut(&mut self) -> &mut EntityBuilder {
        &mut self.builder
    }
}

impl<B: DynamicBundle, T: Component> From<B> for TreeBuilder<T> {
    fn from(bundle: B) -> Self {
        let mut builder = EntityBuilder::new();
        builder.add_bundle(bundle);

        Self {
            children: Vec::new(),
            builder,
            marker: PhantomData,
        }
    }
}

impl<T: Component> From<TreeBuilderClone<T>> for TreeBuilder<T> {
    fn from(tree: TreeBuilderClone<T>) -> Self {
        let mut builder = EntityBuilder::new();
        builder.add_bundle(&tree.builder.build());

        let children = tree
            .children
            .into_iter()
            .map(|child| child.into())
            .collect();

        Self {
            children,
            builder,
            marker: PhantomData,
        }
    }
}

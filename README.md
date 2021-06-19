# hecs-hierarchy
Hierarchy implementation for hecs ECS.

## Features
- [X] Iterate children of parent
- [X] Lookup parent of child
- [X] Traverse hierarchy depth first
- [X] Traverse hierarchy breadth first
- [X] Traverse ancestors
- [X] Detach child from hierarchy
- [ ] Reverse iteration
- [ ] Sorting
- [ ] (Optional) associated data to relation

## Usage
<!-- cargo-sync-readme start -->

hecs-hierarchy adds a hierarchy implementation to hecs ecs.

Import the `Hierarchy` which extends [hecs::World](hecs::World)

The trait [Hierarchy](hecs_hierarchy::Hierarchy) extends [hecs::World](hecs::World) with functions for
manipulating and iterating the hierarchy tree.

The hierarchy uses a marker type which makes it possible for a single entity to belong to
several hierarchy trees.

Example usage:
```rust
use hecs_hierarchy::Hierarchy;

// Marker type which allows several hierarchies.
struct Tree;

let mut world = hecs::World::default();

// Create a root entity, there can be several.
let root = world.spawn(("Root",));

// Create a loose entity
let child = world.spawn(("Child 1",));

// Attaches the child to a parent, in this case `root`
world.attach::<Tree>(child, root).unwrap();

// Iterate children
for child in world.children::<Tree>(root) {
    let name = world.get::<&str>(child).unwrap();
    println!("Child: {:?} {}", child, *name);
}

// Add a grandchild
world.attach_new::<Tree, _>(child, ("Grandchild",)).unwrap();

// Iterate recursively
for child in world.descendants_depth_first::<Tree>(root) {
    let name = world.get::<&str>(child).unwrap();
    println!("Child: {:?} {}", child, *name)
}

// Detach `child` and `grandchild`
world.detach::<Tree>(child).unwrap();

let child2 = world.attach_new::<Tree, _>(root, ("Child 2",)).unwrap();

// Reattach as a child of `child2`
world.attach::<Tree>(child, child2).unwrap();

world.attach_new::<Tree, _>(root, ("Child 3",)).unwrap();

// Hierarchy now looks like this:
// Root
// |-------- Child 3
// |-------- Child 2
//           |-------- Child 1
//                     |-------- Grandchild

```

<!-- cargo-sync-readme end -->

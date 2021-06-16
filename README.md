# hecs-hierarchy
Hierarchy implementation for hecs ECS.

## Features
- Lookup children for entity
- Lookup parent for entity
- Traverse hierarchy depth first
- Traverse hierarchy breadth first

## Usage
```rust
// Marker type which allows several hierarchies.
struct Tree;

let world = hecs::World::default();
let root = world.spawn_one("Root")

let child = world.spawn_one("Child 1")

// Attaches the child to a parent, in this case `root`
world.attach::<Tree>(child, root)

// Iterate children
for child in world.children::<Tree>(root) {
  println!("Child: {:?} {}", child, name)
}
```

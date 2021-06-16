# hecs-hierarchy
Hierarchy implementation for hecs ECS.

## Features
- [X] Iterate children of parent
- [ ] Lookup parent of child
- [ ] Traverse hierarchy depth first
- [ ] Traverse hierarchy breadth first
- [ ] Traverse ancestors
- [ ] Detach child from hierarchy
- [ ] Reverse iteration
- [ ] Sorting
- [ ] (Optional) associated data to relation

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

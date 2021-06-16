use std::collections::HashSet;

use hecs::{Entity, World};
use hecs_hierarchy::Hierarchy;

struct Tree;

#[test]
fn basic() {
    let mut world = World::default();
    let root = world.spawn(("Root",));

    let child1 = world.spawn(("Child 1",));
    let child2 = world.spawn(("Child 2",));

    // Attaches the child to a parent, in this case `root`
    world.attach::<Tree>(child1, root).unwrap();
    world.attach::<Tree>(child2, root).unwrap();

    // Make sure Hierarchy is correct but don't care about order.
    let mut expected_childern: HashSet<Entity> = [child1, child2].iter().cloned().collect();

    for child in world.children::<Tree>(root) {
        let name = world.get::<&str>(child).unwrap();

        if !expected_childern.remove(&child) {
            panic!("Entity {:?} does not belong in hierarchy", child);
        }
        println!("Child: {:?} {:?}", child, *name);
    }

    if !expected_childern.is_empty() {
        panic!("Not all children in hierarchy were visited")
    }
}

#[test]
fn empty() {
    let mut world = World::default();

    let empty_root = world.spawn(("Root",));

    assert_eq!(
        world
            .children::<Tree>(empty_root)
            .map(|child| println!("Entity {:?} does not belong in hierarchy", child))
            .count(),
        0
    )
}

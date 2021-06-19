use std::collections::HashSet;

use hecs::{Entity, World};
use hecs_hierarchy::{Child, Hierarchy};

#[derive(Debug)]
struct Tree;

#[test]
fn basic() {
    let mut world = World::default();
    let root = world.spawn(("Root",));

    // Attaches the child to a parent, in this case `root`
    let child_count = 10;

    // Make sure Hierarchy is correct but don't care about order.
    let mut expected_childern: HashSet<Entity> = HashSet::new();

    for i in 0..child_count {
        let child = world.spawn((format!("Child {}", i),));
        expected_childern.insert(child);
        world.attach::<Tree>(child, root).unwrap();
    }

    for child in world.children::<Tree>(root) {
        let name = world.get::<String>(child).unwrap();

        if !expected_childern.remove(&child) {
            panic!("Entity {:?} does not belong in hierarchy", child);
        }

        println!(
            "Child: {:?} {:?}; {:?}",
            child,
            *name,
            *world.get::<Child<Tree>>(child).unwrap()
        );
    }

    if !expected_childern.is_empty() {
        panic!("Not all children in hierarchy were visited")
    }
}

#[test]
fn ancestors() {
    let mut world = World::default();
    let depth = 10;
    let root = world.spawn((String::from("Root"),));

    let mut children = vec![root];

    for i in 1..depth {
        let child = world.spawn((format!("Child {}", i),));
        world.attach::<Tree>(child, children[i - 1]).unwrap();
        children.push(child);
    }

    assert!(world
        .ancestors::<Tree>(children.pop().unwrap())
        .map(|parent| {
            println!("{}", *world.get::<String>(parent).unwrap());
            parent
        })
        .eq(children.into_iter().rev()));
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

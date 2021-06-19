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
fn detach() {
    // Root ---- Child 1
    //      ---- Child 2
    //           ------- Child 3
    //                   ------- Child 4

    let mut world = World::default();
    let root = world.spawn(("Root",));
    let child1 = world.attach_new::<Tree, _>(root, ("Child1",)).unwrap();
    let child2 = world.attach_new::<Tree, _>(root, ("Child2",)).unwrap();
    let _child3 = world.attach_new::<Tree, _>(child2, ("Child3",)).unwrap();
    let child4 = world.attach_new::<Tree, _>(root, ("Child4",)).unwrap();
    let child5 = world.attach_new::<Tree, _>(root, ("Child5",)).unwrap();

    // Remove child2, and by extension child3
    world.detach::<Tree>(child2).unwrap();

    let order = [child5, child4, child1];

    for child in world.children::<Tree>(root) {
        println!(
            "{:?}, {:?}",
            *world.get::<&str>(child).unwrap(),
            *world.get::<Child<Tree>>(child).unwrap()
        );
    }

    assert!(world.children::<Tree>(root).eq(order.iter().cloned()))
}

#[test]
fn reattach() {
    // Root ---- Child 1
    //      ---- Child 2
    //           ------- Child 3
    //                   ------- Child 4

    let mut world = World::default();
    let root = world.spawn(("Root",));
    let child1 = world.attach_new::<Tree, _>(root, ("Child1",)).unwrap();
    let child2 = world.attach_new::<Tree, _>(root, ("Child2",)).unwrap();
    let _child3 = world.attach_new::<Tree, _>(child2, ("Child3",)).unwrap();
    let child4 = world.attach_new::<Tree, _>(root, ("Child4",)).unwrap();
    let child5 = world.attach_new::<Tree, _>(root, ("Child5",)).unwrap();

    // Remove child2, and by extension child3
    world.detach::<Tree>(child2).unwrap();

    // Reattach child2 and child3 under child4
    world.attach::<Tree>(child2, child4).unwrap();

    let order = [child5, child4, child1];

    for child in world.descendants_depth_first::<Tree>(root) {
        println!(
            "{:?}, {:?}",
            *world.get::<&str>(child).unwrap(),
            *world.get::<Child<Tree>>(child).unwrap()
        );
    }

    assert!(world.children::<Tree>(root).eq(order.iter().cloned()))
}

#[test]
fn dfs() {
    // Root ---- Child 1
    //      ---- Child 2
    //           ------- Child 3
    //                   ------- Child 4

    let mut world = World::default();
    let root = world.spawn(("Root",));
    let child1 = world.attach_new::<Tree, _>(root, ("Child1",)).unwrap();
    let child2 = world.attach_new::<Tree, _>(root, ("Child2",)).unwrap();
    let child3 = world.attach_new::<Tree, _>(child2, ("Child3",)).unwrap();
    let child4 = world.attach_new::<Tree, _>(child3, ("Child4",)).unwrap();

    let order = [child2, child3, child4, child1];

    for child in world.descendants_depth_first::<Tree>(root) {
        println!("{:?}", *world.get::<&str>(child).unwrap());
    }

    assert!(world
        .descendants_depth_first::<Tree>(root)
        .eq(order.iter().cloned()))
}

#[test]
fn bfs() {
    // Root ---- Child 1
    //      ---- Child 2
    //           ------- Child 3
    //                   ------- Child 4

    let mut world = World::default();
    let root = world.spawn(("Root",));
    let child1 = world.attach_new::<Tree, _>(root, ("Child1",)).unwrap();
    let child2 = world.attach_new::<Tree, _>(root, ("Child2",)).unwrap();
    let child3 = world.attach_new::<Tree, _>(child2, ("Child3",)).unwrap();
    let child4 = world.attach_new::<Tree, _>(child3, ("Child4",)).unwrap();

    let order = [child2, child1, child3, child4];

    for child in world.descendants_breadth_first::<Tree>(root) {
        println!("{:?}", *world.get::<&str>(child).unwrap());
    }

    assert!(world
        .descendants_breadth_first::<Tree>(root)
        .eq(order.iter().cloned()))
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

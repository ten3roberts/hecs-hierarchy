use std::collections::HashSet;

use hecs::{Entity, World};
use hecs_hierarchy::{Child, Hierarchy, HierarchyMut, HierarchyQuery, TreeBuilder};
use hecs_schedule::{GenericWorld, SubWorldRef};

#[derive(Debug)]
struct Tree;

#[test]
fn basic() {
    let mut world = World::default();
    let root = world.spawn(("Root",));

    // Attaches the child to a parent, in this case `root`
    let child_count = 10;

    // Make sure Hierarchy is correct but don't care about order.
    let mut expected_children: HashSet<Entity> = HashSet::new();

    for i in 0..child_count {
        let child = world.spawn((format!("Child {}", i),));
        expected_children.insert(child);
        world.attach::<Tree>(child, root).unwrap();
    }

    for child in world.children::<Tree>(root) {
        let name = world.get::<String>(child).unwrap();

        println!(
            "Child: {:?} {:?}; {:?}",
            child,
            *name,
            *world.get::<Child<Tree>>(child).unwrap()
        );

        if !expected_children.remove(&child) {
            panic!("Entity {:?} does not belong in hierarchy", child);
        }
    }

    if !expected_children.is_empty() {
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

    assert_eq!(
        world
            .ancestors::<Tree>(children.pop().unwrap())
            .map(|parent| {
                println!("{}", *world.get::<String>(parent).unwrap());
                parent
            })
            .collect::<Vec<_>>(),
        children.into_iter().rev().collect::<Vec<_>>()
    );
}

#[test]
fn detach() {
    // Root ---- Child 1
    //      ---- Child 2
    //           ------- Child 3
    //      ---- Child 4
    //      ---- Child 5

    let mut world = World::default();
    let root = world.spawn(("Root",));
    let child1 = world.attach_new::<Tree, _>(root, ("Child1",)).unwrap();
    let child2 = world.attach_new::<Tree, _>(root, ("Child2",)).unwrap();
    let _child3 = world.attach_new::<Tree, _>(child2, ("Child3",)).unwrap();
    let child4 = world.attach_new::<Tree, _>(root, ("Child4",)).unwrap();
    let child5 = world.attach_new::<Tree, _>(root, ("Child5",)).unwrap();

    // Remove child2, and by extension child3
    world.detach::<Tree>(child2).unwrap();

    let order = [child1, child4, child5];

    for child in world.children::<Tree>(root) {
        println!(
            "{:?}, {:?}",
            *world.get::<&str>(child).unwrap(),
            *world.get::<Child<Tree>>(child).unwrap()
        );
    }

    assert_eq!(
        world.children::<Tree>(root).collect::<Vec<_>>(),
        order.iter().cloned().collect::<Vec<_>>()
    );
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

    let order = [child1, child4, child5];

    for child in world.descendants_depth_first::<Tree>(root) {
        println!(
            "{:?}, {:?}",
            *world.get::<&str>(child).unwrap(),
            *world.get::<Child<Tree>>(child).unwrap()
        );
    }

    assert_eq!(
        world.children::<Tree>(root).collect::<Vec<_>>(),
        order.iter().cloned().collect::<Vec<_>>()
    );
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

    let order = [child1, child2, child3, child4];

    for child in world.descendants_depth_first::<Tree>(root) {
        println!("{:?}", *world.get::<&str>(child).unwrap());
    }

    assert_eq!(
        world
            .descendants_depth_first::<Tree>(root)
            .collect::<Vec<_>>(),
        order.iter().cloned().collect::<Vec<_>>()
    );
}

#[test]
fn dfs_skip() {
    // Root ---- Child 1
    //      ---- Child 2
    //           ------- Child 3
    //                   ------- Child 4

    struct Skip;

    let mut world = World::default();
    let root = world.spawn(("Root",));
    let child1 = world.attach_new::<Tree, _>(root, ("Child1",)).unwrap();
    let child2 = world.attach_new::<Tree, _>(root, ("Child2",)).unwrap();
    let child3 = world
        .attach_new::<Tree, _>(child2, ("Child3", Skip))
        .unwrap();
    let _child4 = world.attach_new::<Tree, _>(child3, ("Child4",)).unwrap();

    let order = [child1, child2];

    for child in world.visit::<Tree, _>(root, |w, e| w.try_get::<Skip>(e).is_err()) {
        println!("{:?}", *world.get::<&str>(child).unwrap());
    }

    assert_eq!(
        world
            .visit::<Tree, _>(root, |w, e| w.try_get::<Skip>(e).is_err())
            .collect::<Vec<_>>(),
        order.iter().cloned().collect::<Vec<_>>()
    );
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

    let order = [child1, child2, child3, child4];

    for child in world.descendants_breadth_first::<Tree>(root) {
        println!("{:?}", *world.get::<&str>(child).unwrap());
    }

    assert_eq!(
        world
            .descendants_breadth_first::<Tree>(root)
            .collect::<Vec<_>>(),
        order.iter().cloned().collect::<Vec<_>>()
    );
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

#[test]
fn roots() {
    let mut world = World::default();
    let root1 = world.spawn(("Root1",));
    let root2 = world.spawn(("Root2",));
    let root3 = world.spawn(("Root3",));

    world.attach_new::<Tree, _>(root1, ("Child1",)).unwrap();
    world.attach_new::<Tree, _>(root1, ("Child2",)).unwrap();
    world.attach_new::<Tree, _>(root2, ("Child3",)).unwrap();
    world.attach_new::<Tree, _>(root1, ("Child4",)).unwrap();
    world.attach_new::<Tree, _>(root3, ("Child5",)).unwrap();

    let mut expected = [root1, root2, root3];
    expected.sort();

    let subworld = SubWorldRef::<HierarchyQuery<Tree>>::new(&world);

    let mut roots = subworld
        .roots::<Tree>()
        .unwrap()
        .iter()
        .map(|(e, _)| e)
        .collect::<Vec<_>>();

    roots.sort();

    dbg!(&roots, &expected);

    assert_eq!(
        roots.iter().collect::<Vec<_>>(),
        expected.iter().collect::<Vec<_>>()
    );
}

#[test]
fn builder() {
    let mut world = World::default();
    let builder = TreeBuilder::<Tree>::new(&mut world);
    let root = builder
        .spawn_tree(("root",))
        .attach_new(("child 1",))
        .attach_new(("child 2",))
        .attach(
            builder
                .spawn_tree(("child 3",))
                .attach_new(("child 3.1",))
                .entity(),
        )
        .entity();

    let expected = ["child 1", "child 2", "child 3", "child 3.1"];

    assert!(world
        .descendants_breadth_first::<Tree>(root)
        .zip(expected)
        .map(|(e, expected)| {
            let name = *world.get::<&str>(e).unwrap();
            eprintln!("Name: {}", name);
            name == expected
        })
        .all(|val| val == true));
}

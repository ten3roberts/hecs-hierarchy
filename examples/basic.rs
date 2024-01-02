use std::error::Error;

use hecs::{Entity, World};
use hecs_hierarchy::*;

fn main() -> Result<(), Box<dyn Error>> {
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
        let name = world.get::<&&str>(child).unwrap();
        println!("Child: {:?} {}", child, *name);
    }

    // Add a grandchild
    world.attach_new::<Tree, _>(child, ("Grandchild",)).unwrap();

    // Iterate recursively
    for child in world.descendants_depth_first::<Tree>(root) {
        let name = world.get::<&&str>(child).unwrap();
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

    print_tree::<Tree>(&world, root);

    world.despawn_all::<Tree>(child2);

    print_tree::<Tree>(&world, root);

    world
        .iter()
        .for_each(|entity| println!("Entity: {:?}", entity.entity()));

    Ok(())
}

fn print_tree<T: 'static + Send + Sync>(world: &World, root: Entity) {
    fn internal<T: 'static + Send + Sync>(world: &World, parent: Entity, depth: usize) {
        for child in world.children::<T>(parent) {
            let name = world.get::<&&str>(child).unwrap();
            println!(
                "{}|-------- {}",
                std::iter::repeat(" ")
                    .take((depth - 1) * 10)
                    .collect::<String>(),
                *name,
            );

            internal::<T>(world, child, depth + 1)
        }
    }

    let name = world.get::<&&str>(root).unwrap();
    println!("{}", *name);
    internal::<T>(world, root, 1)
}

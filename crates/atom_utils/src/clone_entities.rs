use std::any::TypeId;
use std::sync::Arc;

use bevy::{
    ecs::{entity::EntityHashMap, query::QueryEntityError, world::Command},
    prelude::*,
};

// Copy all components from an entity to another.
// Using an entity with no components as the destination creates a copy of the source entity.
// panics if
// - the components are not registered in the type registry,
// - the world does not have a type registry
// - the source or destination entity do not exist
fn clone_entity_components(world: &mut World, source: Entity, destination: Entity) {
    let wc = world.as_unsafe_world_cell();
    unsafe {
        let registry = wc.world().get_resource::<AppTypeRegistry>().unwrap();
        let type_registry = registry.read();

        let entity_ref = wc.world().get_entity(source).unwrap();
        let archetype = entity_ref.archetype();
        let components = wc.world().components();

        for component_id in archetype.components() {
            let component_info = components.get_info(component_id).unwrap();
            let type_id = component_info.type_id().unwrap();
            if TypeId::of::<Children>() == type_id || TypeId::of::<Parent>() == type_id {
                continue;
            }
            let registration = type_registry.get(type_id).unwrap_or_else(|| {
                panic!(
                    "expected {} to be registered to type registry",
                    component_info.name()
                )
            });
            let reflect_component = registration.data::<ReflectComponent>().unwrap_or_else(|| {
                panic!(
                    "expected {} to have a ReflectComponent, add #[reflect(Component)] to the component",
                    component_info.name()
                )
            });
            // here is where the magic happens
            reflect_component.copy(
                wc.world_mut(),
                wc.world_mut(),
                source,
                destination,
                &type_registry,
            );
        }
    }
}

/// Clone entity including their children.
fn clone_entity_recursive(world: &mut World, source: Entity, destination: Entity) {
    clone_entity_components(world, source, destination);
    let source = world.get_entity(source).unwrap();
    let source_children = match source.get::<Children>() {
        Some(o) => o,
        None => return,
    };
    let source_children: Vec<Entity> = source_children.iter().copied().collect();
    for child in source_children {
        let child_cloned = world.spawn_empty().id();
        clone_entity_recursive(world, child, child_cloned);
        world
            .get_entity_mut(destination)
            .unwrap()
            .add_child(child_cloned);
    }
}

// this allows the command to be used in systems
pub struct CloneEntityCommand(pub Entity, pub Entity);

impl Command for CloneEntityCommand {
    fn apply(self, world: &mut World) {
        clone_entity_components(world, self.0, self.1);
    }
}

pub struct CloneEntityRecursiveCommand(pub Entity, pub Entity);
impl Command for CloneEntityRecursiveCommand {
    fn apply(self, world: &mut World) {
        clone_entity_recursive(world, self.0, self.1);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityTreeNode {
    pub source: Entity,
    pub destination: Entity,
    pub children: Vec<EntityTreeNode>,
}

impl EntityTreeNode {
    // 构造方法
    pub fn from_entity_recursive(
        commands: &mut Commands,
        source_entity: Entity,
        destination_entity: Option<Entity>,
        q_children: &Query<&Children>,
    ) -> EntityTreeNode {
        let children = match q_children.get(source_entity) {
            Ok(children) => children
                .iter()
                .map(|&child| {
                    EntityTreeNode::from_entity_recursive(commands, child, None, q_children)
                })
                .collect::<Vec<_>>(),
            Err(QueryEntityError::QueryDoesNotMatch(_)) => vec![],
            Err(e) => panic!("{}", e),
        };
        EntityTreeNode {
            source: source_entity,
            destination: match destination_entity {
                Some(dest) => dest,
                None => commands.spawn_empty().id(),
            },
            children,
        }
    }

    pub fn iter_children(&self) -> impl Iterator<Item = &EntityTreeNode> {
        self.children.iter()
    }

    pub fn recursive_get_destination_entities(&self) -> Vec<Entity> {
        let mut entities = vec![self.destination];
        for child in self.iter_children() {
            entities.extend(child.recursive_get_destination_entities());
        }
        entities
    }

    pub fn recursive_get_entities_map(&self) -> EntityHashMap<Entity> {
        let mut entities = EntityHashMap::default();
        entities.insert(self.source, self.destination);
        for child in self.iter_children() {
            entities.extend(child.recursive_get_entities_map());
        }
        entities
    }
}

fn clone_entity_tree(
    world: &mut World,
    EntityTreeNode {
        source,
        destination,
        children,
    }: &EntityTreeNode,
) {
    clone_entity_components(world, *source, *destination);
    for node in children {
        clone_entity_tree(world, node);
        let mut destination = world.get_entity_mut(*destination).unwrap();
        destination.add_child(node.destination);
    }
}

// uses arc to prevent cloning the whole tree.
pub struct CloneEntityTreeCommand(pub Arc<EntityTreeNode>);

impl Command for CloneEntityTreeCommand {
    fn apply(self, world: &mut World) {
        clone_entity_tree(world, &self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Reflect, Component, Debug, PartialEq, Eq)]
    #[reflect(Component)]
    pub struct A {
        pub value: usize,
        pub name: String,
    }

    #[derive(Reflect, Component, Debug, PartialEq)]
    #[reflect(Component)]
    pub struct B {
        pub value: f32,
        pub name: String,
    }

    #[derive(Reflect, Component, Debug, PartialEq, Eq)]
    #[reflect(Component)]
    pub struct C {
        pub value: (i32, i32),
        pub name: String,
    }

    #[test]
    fn test_clone_entity_components() {
        let mut world = World::default();
        let registry = AppTypeRegistry::default();
        {
            let mut type_registry = registry.write();
            type_registry.register::<A>();
            type_registry.register::<B>();
            type_registry.register::<C>();
        }
        world.insert_resource(registry);

        let entity1 = world
            .spawn((
                B {
                    value: 1.0,
                    name: "B".to_string(),
                },
                C {
                    value: (2, 10),
                    name: "C".to_string(),
                },
            ))
            .id();
        let entity2 = world
            .spawn(A {
                value: 3,
                name: "A".to_string(),
            })
            .id();

        clone_entity_components(&mut world, entity1, entity2);

        // Check if components are cloned correctly
        assert_eq!(
            world.get::<A>(entity2),
            Some(&A {
                value: 3,
                name: "A".to_string(),
            })
        );
        assert_eq!(
            world.get::<B>(entity2),
            Some(&B {
                value: 1.0,
                name: "B".to_string(),
            })
        );
        assert_eq!(
            world.get::<C>(entity2),
            Some(&C {
                value: (2, 10),
                name: "C".to_string(),
            })
        );
    }
}

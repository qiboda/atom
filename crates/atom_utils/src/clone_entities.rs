use std::sync::Arc;

use bevy::{ecs::entity::EntityHashMap, prelude::*};

/// Entity 树节点，记录源 entity 和克隆后的新 entity
#[derive(Debug)]
pub struct EntityTreeNode {
    /// 源 entity
    pub source: Entity,
    /// 克隆后的新 entity
    pub target: Entity,
    /// 子节点
    pub children: Vec<EntityTreeNode>,
}

impl EntityTreeNode {
    /// 从源 entity 递归构建 entity 树。
    /// 为每个节点 spawn 一个空 entity（root 可指定已有 entity）。
    pub fn from_entity_recursive(
        commands: &mut Commands,
        source: Entity,
        target_override: Option<Entity>,
        children_query: &Query<&Children>,
    ) -> Self {
        let target = target_override.unwrap_or_else(|| commands.spawn_empty().id());

        let children = if let Ok(children) = children_query.get(source) {
            children
                .iter()
                .map(|child| Self::from_entity_recursive(commands, child, None, children_query))
                .collect()
        } else {
            Vec::new()
        };

        EntityTreeNode {
            source,
            target,
            children,
        }
    }

    /// 递归获取 old→new entity 映射
    pub fn recursive_get_entities_map(&self) -> EntityHashMap<Entity> {
        let mut map = EntityHashMap::default();
        self.collect_entities_map(&mut map);
        map
    }

    fn collect_entities_map(&self, map: &mut EntityHashMap<Entity>) {
        map.insert(self.source, self.target);
        for child in &self.children {
            child.collect_entities_map(map);
        }
    }
}

/// 克隆 entity 树中所有源 entity 的组件到对应的目标 entity
pub struct CloneEntityTreeCommand(pub Arc<EntityTreeNode>);

impl Command for CloneEntityTreeCommand {
    fn apply(self, world: &mut World) {
        clone_node_recursive(world, &self.0);
    }
}

fn clone_node_recursive(world: &mut World, node: &EntityTreeNode) {
    // 使用 Bevy 的 entity cloning 功能克隆所有组件
    if world.get_entity(node.source).is_ok() && world.get_entity(node.target).is_ok() {
        world
            .entity_mut(node.source)
            .clone_with_opt_out(node.target, |_| {});
    }

    // 设置父子关系
    for child in &node.children {
        if world.get_entity(child.target).is_ok() {
            world
                .entity_mut(child.target)
                .set_parent_in_place(node.target);
        }
    }

    // 递归处理子节点
    for child in &node.children {
        clone_node_recursive(world, child);
    }
}

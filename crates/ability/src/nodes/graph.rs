use super::node::EffectNode;

pub struct EffectNodeGraph {
    pub all_effect_nodes: Vec<dyn EffectNode>,
}

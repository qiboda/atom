pub mod bundle;
pub mod comp;
pub mod event;
pub mod layertag;
pub mod node;
pub mod plugin;

// 主动和被动技能不需要区分，因为技能可以根据是否是用户特定的事件来触发来决定。

// 技能节点应该有一个初始化阶段，用来初始化技能的各种属性，比如技能的冷却时间，技能的消耗, 是否是主动技能，技能攻击范围等内容。

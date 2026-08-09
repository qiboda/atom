//! 节点引脚定义：执行口（exec）与数据槽（slot）的命名与类型描述。

use std::any::TypeId;

use bevy::prelude::*;

/// 一组引脚：一个执行口及其附属的数据槽。
#[derive(Debug)]
pub struct EffectNodeExecGroup {
    /// 执行口。
    pub exec: EffectNodeExec,
    /// 该执行口携带的数据槽列表。
    pub slots: Vec<EffectNodeSlot>,
}

/// 执行口：以静态字符串命名的执行连线端点。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Reflect)]
pub struct EffectNodeExec {
    /// 执行口名称。
    pub name: &'static str,
}

impl From<&'static str> for EffectNodeExec {
    fn from(name: &'static str) -> Self {
        Self { name }
    }
}

/// 数据槽：以名称 + 数据类型标识的数据连线端点。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub struct EffectNodeSlot {
    /// 数据槽名称。
    pub name: &'static str,
    /// 数据类型标识（运行时校验槽值类型）。
    pub pin_type: TypeId,
}

impl EffectNodeSlot {
    /// 构造类型为 `T` 的数据槽。
    pub fn new<T: 'static>(name: &'static str) -> Self {
        Self {
            name,
            pin_type: TypeId::of::<T>(),
        }
    }
}

/// 节点引脚组 trait：描述节点的输入/输出执行口与数据槽。
#[reflect_trait]
pub trait EffectNodePinGroup {
    /// 返回输入引脚组列表。
    fn get_input_pin_group(&self) -> &Vec<EffectNodeExecGroup>;

    /// 按执行口名称查找输入引脚组。
    fn get_input_pin_group_by_name(&self, name: &str) -> Option<&EffectNodeExecGroup> {
        self.get_input_pin_group()
            .iter()
            .find(|group| group.exec.name == name)
    }

    /// 按名称查找输入执行口。
    fn get_input_exec_pin_by_name(&self, name: &str) -> Option<&EffectNodeExec> {
        for group in self.get_input_pin_group() {
            if group.exec.name == name {
                return Some(&group.exec);
            }
        }
        None
    }

    /// 按名称查找输入数据槽。
    fn get_input_slot_pin_by_name(&self, name: &str) -> Option<&EffectNodeSlot> {
        for group in self.get_input_pin_group() {
            for slot in &group.slots {
                if slot.name == name {
                    return Some(slot);
                }
            }
        }
        None
    }

    /// 输入执行口数量。
    fn get_input_pin_group_num(&self) -> usize {
        self.get_input_pin_group().len()
    }

    /// 返回输出引脚组列表。
    fn get_output_pin_group(&self) -> &Vec<EffectNodeExecGroup>;

    /// 按执行口名称查找输出引脚组。
    fn get_output_pin_group_by_name(&self, name: &str) -> Option<&EffectNodeExecGroup> {
        self.get_output_pin_group()
            .iter()
            .find(|group| group.exec.name == name)
    }

    /// 按名称查找输出执行口。
    fn get_output_exec_pin_by_name(&self, name: &str) -> Option<&EffectNodeExec> {
        for group in self.get_output_pin_group() {
            if group.exec.name == name {
                return Some(&group.exec);
            }
        }
        None
    }

    /// 按名称查找输出数据槽。
    fn get_output_slot_pin_by_name(&self, name: &str) -> Option<&EffectNodeSlot> {
        for group in self.get_output_pin_group() {
            for slot in &group.slots {
                if slot.name == name {
                    return Some(slot);
                }
            }
        }
        None
    }

    /// 输出执行口数量。
    fn get_output_pin_group_num(&self) -> usize {
        self.get_output_pin_group().len()
    }
}

/// 为节点类型实现 [`EffectNodePinGroup`] 的宏。
///
/// 用法：`impl_effect_node_pin_group!(Node, input => (exec => (slot1: Type1)), output => (exec => ()))`。
/// 会为节点生成输入/输出执行口与数据槽的名称常量（`INPUT_EXEC_*`/`INPUT_SLOT_*`/`OUTPUT_EXEC_*`/`OUTPUT_SLOT_*`），
/// 并实现引脚组查询。
#[macro_export]
macro_rules! impl_effect_node_pin_group {
    ($node:ty) => {
        impl_effect_node_pin_group!($node, input => () output => ());
    };
    ($node:ty, output => ($($out_exec:ident => ($($out_pin:ident: $out_type:ty), *)), +) ) => {
        impl_effect_node_pin_group!($node, input => () output => ($($out_exec => ($($out_pin: $out_type), *)), +));
    };
    ($node:ty, input => ($($in_exec:ident => ($($in_pin:ident: $in_type:ty), *)), +) ) => {
        impl_effect_node_pin_group!($node, input => ($($in_exec => ($($in_pin: $in_type), *)), +) output => ());
    };
    ($node:ty, input => ($($in_exec:ident => ($($in_pin:ident: $in_type:ty), *)), *) output => ($($out_exec:ident => ($($out_pin:ident: $out_type:ty), *)), *)) => {

        impl $node {
            $(
                paste::paste! {
                    /// 输入执行口名称常量。
                    pub const [<INPUT_EXEC_ $in_exec:snake:upper>]: &'static str = stringify!($in_exec);
                    $(
                        /// 输入数据槽名称常量。
                        pub const [<INPUT_SLOT_ $in_pin:snake:upper>]: &'static str = stringify!($in_pin);
                    )*
                }
            )*

            $(
                paste::paste! {
                    /// 输出执行口名称常量。
                    pub const [<OUTPUT_EXEC_ $out_exec:snake:upper>]: &'static str = stringify!($out_exec);
                    $(
                        /// 输出数据槽名称常量。
                        pub const [<OUTPUT_SLOT_ $out_pin:snake:upper>]: &'static str = stringify!($out_pin);
                    )*
                }
            )*
        }

        impl $crate::graph::node::pin::EffectNodePinGroup for $node {
            fn get_input_pin_group(&self) -> &Vec<$crate::graph::node::pin::EffectNodeExecGroup> {
                static CELL: once_cell::sync::OnceCell<Vec<$crate::graph::node::pin::EffectNodeExecGroup>> = once_cell::sync::OnceCell::new();
                CELL.get_or_init(|| {
                    vec![
                    $(
                        paste::paste! {
                            $crate::graph::node::pin::EffectNodeExecGroup {
                                exec: $crate::graph::node::pin::EffectNodeExec { name: $node::[<INPUT_EXEC_ $in_exec:snake:upper>] },
                                slots: vec![
                                $(
                                    $crate::graph::node::pin::EffectNodeSlot {
                                        name: $node::[<INPUT_SLOT_ $in_pin:snake:upper>],
                                        pin_type: std::any::TypeId::of::<$in_type>(),
                                    },
                                )*
                                ],
                            }
                        },
                    )*
                    ]
                })
            }

            fn get_output_pin_group(&self) -> &Vec<$crate::graph::node::pin::EffectNodeExecGroup> {
                static CELL: once_cell::sync::OnceCell<Vec<$crate::graph::node::pin::EffectNodeExecGroup>> = once_cell::sync::OnceCell::new();
                CELL.get_or_init(|| {
                    vec![
                    $(
                        paste::paste! {
                            $crate::graph::node::pin::EffectNodeExecGroup {
                                exec: $crate::graph::node::pin::EffectNodeExec { name: $node::[<OUTPUT_EXEC_ $out_exec:snake:upper>] },
                                slots: vec![
                                    $(
                                        $crate::graph::node::pin::EffectNodeSlot {
                                            name: $node::[<OUTPUT_SLOT_ $out_pin:snake:upper>],
                                            pin_type: std::any::TypeId::of::<$out_type>(),
                                        },
                                    )*
                                ],
                            }
                        },
                    )*
                    ]
                })
            }
        }
    };
}

#[allow(dead_code)]
#[cfg(test)]
mod tests {
    use super::*;

    struct EffectNodeInput;

    impl_effect_node_pin_group!(EffectNodeInput, input => (
            exec => (
                start :i32, duration: f32
            )
        )
    );

    struct EffectNodeInputNone;

    impl_effect_node_pin_group!(EffectNodeInputNone, input => (
            exec => ()
        )
    );

    struct EffectNodeOutput;

    impl_effect_node_pin_group!(EffectNodeOutput, output => (
            exec => (
                start :i32, duration: f32
            )
        )
    );

    struct EffectNodeOutputNone;

    impl_effect_node_pin_group!(EffectNodeOutputNone, input => (
            exec => ()
        )
    );

    struct EffectNodeNone;

    impl_effect_node_pin_group!(EffectNodeNone);

    #[test]
    fn input_pin_group_query_by_name() {
        let node = EffectNodeInput;
        assert_eq!(node.get_input_pin_group_num(), 1);

        let group = node
            .get_input_pin_group_by_name("exec")
            .expect("exec 输入组应存在");
        assert_eq!(group.exec.name, "exec");
        assert_eq!(group.slots.len(), 2);

        let start = node
            .get_input_slot_pin_by_name("start")
            .expect("start 槽应存在");
        assert_eq!(start.pin_type, TypeId::of::<i32>());

        let duration = node
            .get_input_slot_pin_by_name("duration")
            .expect("duration 槽应存在");
        assert_eq!(duration.pin_type, TypeId::of::<f32>());
    }

    #[test]
    fn output_pin_group_query_by_name() {
        let node = EffectNodeOutput;
        assert_eq!(node.get_output_pin_group_num(), 1);

        let group = node
            .get_output_pin_group_by_name("exec")
            .expect("exec 输出组应存在");
        assert_eq!(group.exec.name, "exec");
        assert_eq!(group.slots.len(), 2);

        let start = node
            .get_output_slot_pin_by_name("start")
            .expect("start 槽应存在");
        assert_eq!(start.pin_type, TypeId::of::<i32>());
    }

    #[test]
    fn pin_group_missing_queries_return_none() {
        let node = EffectNodeInput;
        assert_eq!(node.get_input_exec_pin_by_name("missing"), None);
        assert_eq!(node.get_input_slot_pin_by_name("missing"), None);
        assert!(node.get_input_pin_group_by_name("missing").is_none());
        assert_eq!(node.get_output_exec_pin_by_name("missing"), None);
        assert_eq!(node.get_output_slot_pin_by_name("missing"), None);
        assert!(node.get_output_pin_group_by_name("missing").is_none());
    }

    #[test]
    fn empty_pin_group_declarations() {
        let node = EffectNodeNone;
        assert_eq!(node.get_input_pin_group_num(), 0);
        assert_eq!(node.get_output_pin_group_num(), 0);

        let node = EffectNodeInputNone;
        assert_eq!(node.get_input_pin_group_num(), 1);
        let group = node.get_input_pin_group().first().expect("应有一个输入组");
        assert!(group.slots.is_empty(), "无槽声明时 slots 应为空");
    }

    #[test]
    fn input_exec_pin_group_equality() {
        let node = EffectNodeInput;
        let exec = node
            .get_input_exec_pin_by_name("exec")
            .expect("exec 执行口应存在");
        assert_eq!(exec, &EffectNodeExec { name: "exec" });
    }
}

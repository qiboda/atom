use std::any::TypeId;

use bevy::{prelude::*, reflect::reflect_trait};
use uuid::Uuid;

pub trait EffectNode {}

#[derive(Debug, Component, Default, Copy, Clone, PartialEq, Eq, Reflect)]
pub enum EffectNodeTickState {
    #[default]
    Ticked,
    Paused,
}

#[derive(Debug, Component, Default, Copy, Clone, PartialEq, Eq, Reflect)]
pub enum EffectNodeExecuteState {
    #[default]
    Idle,
    Actived,
}

/// use for deserialize and serialize
#[derive(Debug, Component, Default, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct EffectNodeUuid {
    pub uuid: Uuid,
}

impl EffectNodeUuid {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }
}

pub struct EffectNodeExecGroup {
    pub exec: EffectNodeExec,
    pub pins: Vec<EffectNodePin>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Reflect)]
pub struct EffectNodeExec {
    pub name: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct EffectNodePin {
    pub name: &'static str,
    #[reflect(ignore)]
    #[reflect(default = "std::any::TypeId::of::<()>")]
    pub pin_type: TypeId,
}

#[reflect_trait]
pub trait EffectNodePinGroup {
    fn get_input_pin_group(&self) -> &Vec<EffectNodeExecGroup>;

    fn get_input_pin_group_by_name(&self, name: &str) -> Option<&EffectNodeExecGroup> {
        self.get_input_pin_group()
            .iter()
            .find(|group| group.exec.name == name)
    }

    fn get_input_pin_group_num(&self) -> usize {
        self.get_input_pin_group().len()
    }

    fn get_output_pin_group(&self) -> &Vec<EffectNodeExecGroup>;

    fn get_output_pin_group_by_name(&self, name: &str) -> Option<&EffectNodeExecGroup> {
        self.get_output_pin_group()
            .iter()
            .find(|group| group.exec.name == name)
    }

    fn get_output_pin_group_num(&self) -> usize {
        self.get_output_pin_group().len()
    }
}

#[macro_export]
macro_rules! impl_effect_node_pin_group {
    ($node:ty) => {
        impl_effect_node_pin_group!($node, input => () output => ());
    };
    ($node:ty, output => ($($out_exec:ident, pins => ($($out_pin:ident: $out_type:ty), *)), +) ) => {
        impl_effect_node_pin_group!($node, input => () output => ($($out_exec, pins => ($($out_pin: $out_type), *)), +));
    };
    ($node:ty, input => ($($in_exec:ident, pins => ($($in_pin:ident: $in_type:ty), *)), +) ) => {
        impl_effect_node_pin_group!($node, input => ($($in_exec, pins => ($($in_pin: $in_type), *)), +) output => ());
    };
    ($node:ty, input => ($($in_exec:ident, pins => ($($in_pin:ident: $in_type:ty), *)), *) output => ($($out_exec:ident, pins => ($($out_pin:ident: $out_type:ty), *)), *)) => {

        impl $node {
            $(
                paste::paste! {
                    pub const [<INPUT_EXEC_ $in_exec:snake:upper>]: &'static str = stringify!($in_exec);
                    $(
                        pub const [<INPUT_PIN_ $in_pin:snake:upper>]: &'static str = stringify!($in_pin);
                    )*
                }
            )*

            $(
                paste::paste! {
                    pub const [<OUTPUT_EXEC_ $out_exec:snake:upper>]: &'static str = stringify!($out_exec);
                    $(
                        pub const [<OUTPUT_PIN_ $out_pin:snake:upper>]: &'static str = stringify!($out_pin);
                    )*
                }
            )*
        }

        impl $crate::graph::node::EffectNodePinGroup for $node {
            fn get_input_pin_group(&self) -> &Vec<$crate::graph::node::EffectNodeExecGroup> {
                static CELL: once_cell::sync::OnceCell<Vec<$crate::graph::node::EffectNodeExecGroup>> = once_cell::sync::OnceCell::new();
                CELL.get_or_init(|| {
                    vec![
                    $(
                        paste::paste! {
                            $crate::graph::node::EffectNodeExecGroup {
                                exec: $crate::graph::node::EffectNodeExec { name: $node::[<INPUT_EXEC_ $in_exec:snake:upper>] },
                                pins: vec![
                                $(
                                    $crate::graph::node::EffectNodePin {
                                        name: $node::[<INPUT_PIN_ $in_pin:snake:upper>],
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

            fn get_output_pin_group(&self) -> &Vec<$crate::graph::node::EffectNodeExecGroup> {
                static CELL: once_cell::sync::OnceCell<Vec<$crate::graph::node::EffectNodeExecGroup>> = once_cell::sync::OnceCell::new();
                CELL.get_or_init(|| {
                    vec![
                    $(
                        paste::paste! {
                            $crate::graph::node::EffectNodeExecGroup {
                                exec: $crate::graph::node::EffectNodeExec { name: $node::[<OUTPUT_EXEC_ $out_exec:snake:upper>] },
                                pins: vec![
                                    $(
                                        $crate::graph::node::EffectNodePin {
                                            name: $node::[<OUTPUT_PIN_ $out_pin:snake:upper>],
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
    struct EffectNodeInput;

    impl_effect_node_pin_group!(EffectNodeInput, input => (
            exec, pins => (
                start :i32, duration: f32
            )
        )
    );

    struct EffectNodeInputNone;

    impl_effect_node_pin_group!(EffectNodeInputNone, input => (
            exec, pins => ()
        )
    );

    struct EffectNodeOutput;

    impl_effect_node_pin_group!(EffectNodeOutput, output => (
            exec, pins => (
                start :i32, duration: f32
            )
        )
    );

    struct EffectNodeOutputNone;

    impl_effect_node_pin_group!(EffectNodeOutputNone, input => (
            exec, pins => ()
        )
    );

    struct EffectNodeNone;

    impl_effect_node_pin_group!(EffectNodeNone);
}

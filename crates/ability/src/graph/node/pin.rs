use std::any::TypeId;

use bevy::prelude::*;

#[derive(Debug)]
pub struct EffectNodeExecGroup {
    pub exec: EffectNodeExec,
    pub slots: Vec<EffectNodeSlot>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Reflect)]
pub struct EffectNodeExec {
    pub name: &'static str,
}

impl From<&'static str> for EffectNodeExec {
    fn from(name: &'static str) -> Self {
        Self { name }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub struct EffectNodeSlot {
    pub name: &'static str,
    pub pin_type: TypeId,
}

impl EffectNodeSlot {
    pub fn new<T: 'static>(name: &'static str) -> Self {
        Self {
            name,
            pin_type: TypeId::of::<T>(),
        }
    }
}

#[reflect_trait]
pub trait EffectNodePinGroup {
    fn get_input_pin_group(&self) -> &Vec<EffectNodeExecGroup>;

    fn get_input_pin_group_by_name(&self, name: &str) -> Option<&EffectNodeExecGroup> {
        self.get_input_pin_group()
            .iter()
            .find(|group| group.exec.name == name)
    }

    fn get_input_exec_pin_by_name(&self, name: &str) -> Option<&EffectNodeExec> {
        for group in self.get_input_pin_group() {
            if group.exec.name == name {
                return Some(&group.exec);
            }
        }
        None
    }

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

    fn get_input_pin_group_num(&self) -> usize {
        self.get_input_pin_group().len()
    }

    fn get_output_pin_group(&self) -> &Vec<EffectNodeExecGroup>;

    fn get_output_pin_group_by_name(&self, name: &str) -> Option<&EffectNodeExecGroup> {
        self.get_output_pin_group()
            .iter()
            .find(|group| group.exec.name == name)
    }

    fn get_output_exec_pin_by_name(&self, name: &str) -> Option<&EffectNodeExec> {
        for group in self.get_output_pin_group() {
            if group.exec.name == name {
                return Some(&group.exec);
            }
        }
        None
    }

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

    fn get_output_pin_group_num(&self) -> usize {
        self.get_output_pin_group().len()
    }
}

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
                    pub const [<INPUT_EXEC_ $in_exec:snake:upper>]: &'static str = stringify!($in_exec);
                    $(
                        pub const [<INPUT_SLOT_ $in_pin:snake:upper>]: &'static str = stringify!($in_pin);
                    )*
                }
            )*

            $(
                paste::paste! {
                    pub const [<OUTPUT_EXEC_ $out_exec:snake:upper>]: &'static str = stringify!($out_exec);
                    $(
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
}

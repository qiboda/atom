use leafwing_input_manager::{input_map::InputMap, Actionlike};

pub trait DefaultInputMap<T: Actionlike> {
    fn default_input_map() -> InputMap<T>;
}

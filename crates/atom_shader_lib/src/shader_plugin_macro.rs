/// 用于宏构建结构体成员。
#[allow(unused_macros)]
macro_rules! impl_extra {
    ( @ $name:ident { } -> ($($result:tt)*) ) => (
        pub struct $name {
            $($result)*
        }
    );

    ( @ $name:ident { $param:ident : Option<$type:ty>, $($rest:tt)* } -> ($($result:tt)*) ) => (
        impl_extra!(@ $name { $($rest)* } -> (
            $($result)*
            pub $param : Option<$type>,
        ));
    );

    ( @ $name:ident { $param:ident : Vec<$type:ty>, $($rest:tt)* } -> ($($result:tt)*) ) => (
        impl_extra!(@ $name { $($rest)* } -> (
            $($result)*
            pub $param : Vec<$type>,
        ));
    );

     ( @ $name:ident { $param:ident : $default:tt, $($rest:tt)* } -> ($($result:tt)*) ) => (
        impl_extra!(@ $name { $($rest)* } -> (
            $($result)*
            pub $param : $default,
        ));
    );

    ( $name:ident { $( $param:ident  ($($type:tt)*) ),* $(,)? } ) => (
        impl_extra!(@ $name { $($param : $($type)*,)* } -> ());
    );
}

#[macro_export]
macro_rules! shaders_plugin {
    (
        $module_name: ident,
        $shader_type:ident,
        ($($member_name:ident -> $shaders_path:expr),*)
    ) => {
        paste::paste! {
            shaders_plugin!(_construct -> [<$module_name $shader_type Shaders>] (
                $($member_name),*
            ));

            #[derive(Debug, Default)]
            pub struct [<$module_name $shader_type ShadersPlugin>];

            impl bevy::app::Plugin for [<$module_name $shader_type ShadersPlugin>] {
                fn build(&self, app: &mut bevy::prelude::App) {
                    use bevy::asset::DirectAssetAccessExt;
                    let world = app.world();
                    app.insert_resource(
                        shaders_plugin!(_init -> world, [<$module_name $shader_type Shaders>] ( $($member_name, $shaders_path),* ))
                    );
                }
            }
        }
    };
    // 参考了impl_extra!宏的实现
    (_init -> $world: ident, $name:ident ( $($member_name: ident, $shaders_path: expr),* ) ) => (
        $name {
            $(
                $member_name: $world.load_asset($shaders_path),
            )*
        }
    );
    // 参考了impl_extra!宏的实现
    (_construct -> $name:ident ( $($member_name: ident),* ) ) => (
        #[derive(Debug, Default, bevy::prelude::Resource)]
        pub struct $name {
            $(
                pub $member_name: bevy::asset::Handle<bevy::prelude::Shader>,

            )*
        }
    )
}

macro_rules! atom_shaders_plugin {
    (
        $shader_type:ident,
        ($($member_name:ident -> $shaders_path:expr),*)
    ) => {
        shaders_plugin!(Atom, $shader_type, ($($member_name -> $shaders_path),*));
    }
}

/// 同一个crate中跨文件共享 macro的方法
pub(crate) use atom_shaders_plugin;

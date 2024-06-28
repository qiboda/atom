/// 用于读取和保存配置
///   配置分级
///   1. 游戏默认配置(读取)
///      固定位置。(Config/) 如何配置热加载: 添加新的AssetSource
///      支持读取，也可以覆盖。
///   2. 用户在运行时修改配置后生成的配置
///       需要能够指定目录。(Saved/Config路径, 默认保存路径)如何配置热加载: 添加新的AssetSource
///       必须能够读取和保存。
///   3. 根据配置路径保存数据, 导出数据到任意地方
///   4. 如何避免保存数据命令比较延迟，导致后续写入数据，导致写入数据错误。写入时复制一份传递到事件中。
///   5. 如何避免读取数据(热加载，手动reload)，设置值，之后读取的配置覆盖了设置的值。(热加载，或者手动加载)
///      设置值之后，立即写数据，这会触发热加载，会进行第二次读取数据的操作，避免新设置的值被之前读取的值覆盖。
///   6. 如何处理热加载， 如果是自己保存的数据，不触发热更新。如果是外部修改，触发热更新。
///      都触发不影响正确性，数据相同，只是读取开销。
///      filename is TypePath::short_type_path() + ".toml"

/// 当前热加载：不支持加载增加的文件。仅仅响应修改的文件。
/// 不支持通过asset去写文件。

/// 添加配置resource：检查文件是否存在，不存在创建读取，存在则读取。
/// 主动修改resource，保存配置。触发热加载。再次修改resource，但因为数据相同，并不再保存。
/// 修改配置路径：（保留暂不实现）
/// 配置移除（不需要）
/// 可选字段（不需要）
/// 添加三个事件：保存配置。加载配置(运行时插入resource使用)。配置修改了的事件(用于热加载的后续逻辑处理)。

/// 由于不支持，运行时添加plugin，运行时添加resource暂不支持。支不支持没有什么大的影响。。。。。
pub mod load;
pub mod persist;
pub mod setting_path;
pub mod toml_diff;

use atom_utils::async_event::AsyncEventPlugin;
use bevy::prelude::*;

use load::{create_game_setting, handle_persist_setting_end_event, SettingUpdateEvent};
use persist::PersistSettingEndEvent;
use serde::{Deserialize, Serialize};
// use bevy_common_assets::toml::TomlAssetPlugin;

use crate::load::{
    refresh_final_settings, start_load_settings, InnerSettingHandle, SettingLoadStageWrap,
};
use crate::persist::{persist, PersistSettingEvent};
use crate::setting_path::SettingsPath;

/// Global settings config for the settings plugin
#[derive(Debug)]
pub struct SettingPlugin<S>
where
    S: Setting,
{
    pub paths: SettingsPath<S>,
}

impl<S> Plugin for SettingPlugin<S>
where
    S: Setting,
{
    fn build(&self, app: &mut App) {
        // let extension = SettingsPath::<S>::extension();

        app
            // .add_plugins(TomlAssetPlugin::<S>::new(&[extension.leak()]))
            .add_plugins(AsyncEventPlugin::<PersistSettingEndEvent<S>>::default())
            .insert_resource(self.paths.clone())
            .init_resource::<InnerSettingHandle<S>>()
            .init_resource::<SettingLoadStageWrap<S>>()
            .init_resource::<S>()
            .add_event::<PersistSettingEvent<S>>()
            .add_event::<SettingUpdateEvent<S>>()
            .add_systems(Startup, create_game_setting::<S>)
            .add_systems(
                PreUpdate,
                (
                    handle_persist_setting_end_event::<S>,
                    start_load_settings::<S>,
                    refresh_final_settings::<S>,
                )
                    .chain(),
            )
            .add_systems(Last, persist::<S>);
    }
}

/// settings limits:
///   1. all fields must be Optional
pub trait Setting:
    Resource + Clone + Serialize + TypePath + Default + for<'a> Deserialize<'a> + Asset
{
}

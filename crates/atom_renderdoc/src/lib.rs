#![deny(missing_docs)]
//! RenderDoc 调试集成：提供 `RenderDocPlugin` 初始化 RenderDoc 并注入 World，支持帧捕获与 Replay UI。

use bevy::{prelude::*, render::renderer::RenderDevice};
use renderdoc::*;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate};

pub use renderdoc;

/// RenderDoc API 版本，固定使用 V141。
pub type RenderDocVersion = V141;

/// RenderDoc 实例类型，作为 NonSend resource 注入 World。
pub type RenderDocResource = RenderDoc<RenderDocVersion>;

/// Bevy 插件：初始化 RenderDoc 并将实例注入 World。
/// 必须添加在 `RenderPlugin` 之前；按 F12 启动 RenderDoc Replay UI。
pub struct RenderDocPlugin;

impl Plugin for RenderDocPlugin {
    fn build(&self, app: &mut App) {
        let has_invalid_setup = app.world().contains_resource::<RenderDevice>();

        if has_invalid_setup {
            app.add_systems(Startup, || {
                error!("RenderDocPlugin needs to be added before RenderPlugin!");
            });
            return;
        }

        match RenderDoc::<RenderDocVersion>::new() {
            Ok(mut rd) => {
                rd.set_capture_file_path_template("saved/renderdoc/bevy_capture");
                rd.mask_overlay_bits(OverlayBits::NONE, OverlayBits::NONE);

                app.world_mut().insert_non_send(rd);
                app.add_systems(Startup, || info!("Initialized RenderDoc successfully!"));
                app.add_systems(Update, trigger_capture);
            }
            Err(e) => {
                app.add_systems(Startup, move || error!("Failed to initialize RenderDoc. Ensure RenderDoc is installed and visible from your $PATH. Error: \"{}\"", e));
            }
        }
    }
}

fn trigger_capture(
    key: ResMut<ButtonInput<KeyCode>>,
    rd: NonSendMut<RenderDocResource>,
    mut replay_pid: Local<usize>,
    mut system: Local<sysinfo::System>,
) {
    // TODO: If a user were to change this hotkey on the RenderDoc instance
    // this could get mismatched.
    if key.just_pressed(KeyCode::F12) {
        // Avoid launching multiple instances of the replay ui
        if system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[Pid::from(*replay_pid)]),
            true,
            ProcessRefreshKind::nothing().with_cpu(),
        ) > 0
        {
            return;
        }

        match rd.launch_replay_ui(true, None) {
            Ok(pid) => {
                // rd.start_frame_capture(std::ptr::null(), std::ptr::null());
                *replay_pid = pid as usize;
                info!("Launching RenderDoc Replay UI");
            }
            Err(e) => error!("Failed to launch RenderDoc Replay UI: {}", e),
        }
    }
}

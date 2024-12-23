use bevy::{prelude::*, render::renderer::RenderDevice};
use renderdoc::*;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate};

pub use renderdoc;

pub type RenderDocVersion = V141;

pub type RenderDocResource = RenderDoc<RenderDocVersion>;

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

                app.world_mut().insert_non_send_resource(rd);
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

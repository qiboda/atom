use bevy::{
    prelude::*,
    render::{render_graph::RenderGraph, RenderApp},
};
// use bevy_mod_debugdump::schedule_graph;

pub struct AtomDebugPlugin;

impl Plugin for AtomDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, save_frame_graph);

        let render_app = app
            .get_sub_app_mut(RenderApp)
            .unwrap_or_else(|_| panic!("no render app"));

        render_app.add_systems(Update, save_render_graph);
    }
}

// schedules is not always exists...........
// #[bevycheck::system]
fn save_frame_graph(world: &World, key: Option<Res<Input<KeyCode>>>, _schedules: Res<Schedules>) {
    if key.is_none() {
        return;
    }
    if key.unwrap().just_pressed(KeyCode::F11) {
        // #[cfg(feature = "trace")]
        let _save_frame_graph = info_span!("save frame graph").entered();

        info!("exectue frame graph system");

        if let Some(sches) = world.get_resource::<Schedules>() {
            for (l, _) in sches.iter() {
                info!("{:?}", l);
            }
        }

        // let schedule = schedules.get(&CoreSchedule::Startup);

        // schedule.graph_mut().initialize(world);
        //let _ = schedule.graph_mut().build_schedule(world.components());
        //
        // if let Some(schedule) = schedule {
        //     let settings = schedule_graph::Settings::default();
        //     let dot_string = schedule_graph::schedule_graph_dot(schedule, world, &settings);
        //
        //     dot_output("main_schedule".to_owned(), dot_string);
        // }
    }
}

// render app without input event...
fn save_render_graph(render_graph: Res<RenderGraph>, key: Option<Res<Input<KeyCode>>>) {
    if key.is_none() {
        return;
    }

    if key.unwrap().just_pressed(KeyCode::F11) {
        info!("exectue render graph system");

        let settings = bevy_mod_debugdump::render_graph::Settings {
            style: bevy_mod_debugdump::render_graph::settings::Style::dark_discord(),
        };

        // bevy_mod_debugdump::print_render_graph(app)
        let dot_string =
            bevy_mod_debugdump::render_graph::render_graph_dot(&render_graph, &settings);

        dot_output("render_graph".to_owned(), dot_string)
    }
}

fn dot_output(png_name: String, dot_content: String) {
    // info!("exectue dot output, file: {}", dot_content);

    let dot_filename = png_name.clone() + ".dot";
    let png_filename = png_name + ".png";
    let _ = std::fs::write(dot_filename.clone(), dot_content);

    let output = std::process::Command::new("dot")
        .arg("-Tpng")
        .args(["-o", &png_filename])
        .arg(&dot_filename)
        .output();

    match output {
        Ok(output) => info!(
            "output main schedule_graph to project root path. {:?}",
            output
        ),
        Err(error) => error!("{:?}", error),
    }
}

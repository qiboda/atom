use terrain::bevy_entry;

// #[cfg(feature = "debug_dump")]
// use {
//     bevy::app::RunFixedMainLoop,
//     bevy::prelude::*,
//     bevy::render::{pipelined_rendering::RenderExtractApp, ExtractSchedule, Render, RenderApp},
//     bevy_mod_debugdump::render_graph,
//     bevy_mod_debugdump::{render_graph_dot, schedule_graph, schedule_graph_dot},
// };

// #[cfg(feature = "debug_dump")]
// fn output_dot_file(filename: &str, dot: &str) {
//     use std::fs::File;
//     use std::io::Write;

//     let workspace_path = std::env::var("ATOM_SAVED_ROOT").unwrap();
//     let workspace_path = std::path::Path::new(&workspace_path);
//     let path = workspace_path.join("graphs");

//     let filename = format!("{}.dot", filename);
//     let filename = path.join(filename);

//     if let Ok(()) = std::fs::create_dir_all(path) {
//         let mut file = File::create(filename).unwrap();
//         file.write_all(dot.as_bytes()).unwrap();
//     }
// }

// #[cfg(feature = "debug_dump")]
// fn debug_dump(app: &mut App) {
//     let schedule_settings = schedule_graph::settings::Settings {
//         ..Default::default()
//     };
//     let startup_dot = schedule_graph_dot(app, Startup, &schedule_settings);
//     output_dot_file("Startup", &startup_dot);
//     let post_startup_dot = schedule_graph_dot(app, PostStartup, &schedule_settings);
//     output_dot_file("PostStartup", &post_startup_dot);
//     let first_dot = schedule_graph_dot(app, First, &schedule_settings);
//     output_dot_file("First", &first_dot);
//     let pre_update_dot = schedule_graph_dot(app, PreUpdate, &schedule_settings);
//     output_dot_file("PreUpdate", &pre_update_dot);
//     let update_dot = schedule_graph_dot(app, Update, &schedule_settings);
//     output_dot_file("Update", &update_dot);
//     let fixed_update_dot = schedule_graph_dot(app, RunFixedMainLoop, &schedule_settings);
//     output_dot_file("FixedUpdate", &fixed_update_dot);
//     // let state_transition_dot = schedule_graph_dot( app, StateTransition, &schedule_settings);
//     // output_dot_file("StateTransition", &state_transition_dot);
//     let post_update_dot = schedule_graph_dot(app, PostUpdate, &schedule_settings);
//     output_dot_file("PostUpdate", &post_update_dot);
//     let last_dot = schedule_graph_dot(app, Last, &schedule_settings);
//     output_dot_file("Last", &last_dot);

//     // TODO: Fix runtime crash error.
//     // let extract_schedule_dot = schedule_graph_dot(
//     //     app.sub_app_mut(RenderExtractApp),
//     //     ExtractSchedule,
//     //     &schedule_settings,
//     // );
//     // output_dot_file("Extract", &extract_schedule_dot);

//     let render_dot = schedule_graph_dot(app.sub_app_mut(RenderApp), Render, &schedule_settings);
//     output_dot_file("Render", &render_dot);

//     let render_settings = render_graph::settings::Settings {
//         ..Default::default()
//     };
//     let render_dot = render_graph_dot(app, &render_settings);
//     output_dot_file("RenderGraph", &render_dot);
// }

fn main() {
    let mut app = bevy_entry();

    // #[cfg(feature = "debug_dump")]
    // debug_dump(&mut app);

    // #[cfg(not(feature = "debug_dump"))]
    app.run();
}

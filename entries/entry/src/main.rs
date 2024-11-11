use std::{fs::File, io::Read};

use bevy::log::warn;
use dotenv::dotenv;
use game::network::{
    app_build::GameBuildApp, client::GameClientPlugin, server::GameServerPlugin,
    shared::GameSharedPlugin,
};
use network::{
    app::{Apps, Cli},
    settings::{read_settings, Settings},
};

#[cfg(feature = "debug_dump")]
pub mod debug_dump;
#[cfg(feature = "debug_dump")]
use crate::debug_dump::debug_dump;

fn main() {
    dotenv().ok();

    let cli = Cli::default();

    let config_root_path = project::project_config_root_path();
    let mut file = File::open(config_root_path.join("settings.ron")).unwrap();

    let mut settings_str = String::new();
    match file.read_to_string(&mut settings_str) {
        Ok(_) => {}
        Err(err) => warn!("read setting file: {:?}", err),
    }
    let settings = read_settings::<Settings>(&settings_str);

    // build the bevy app (this adds common plugin such as the DefaultPlugins)
    // and returns the `ClientConfig` and `ServerConfig` so that we can modify them if needed
    let mut apps = Apps::new::<GameBuildApp>(settings, cli);

    apps.update_lightyear_client_config(|x| {
        x.prediction.minimum_input_delay_ticks = 10;
        x.prediction.maximum_input_delay_before_prediction = 20;
        x.prediction.maximum_predicted_ticks = 10;
        x.prediction.correction_ticks_factor = 2.0;
    });
    // add the `ClientPlugins` and `ServerPlugins` plugin groups
    apps.add_lightyear_plugins()
        // add our plugins
        .add_user_plugins(GameClientPlugin, GameServerPlugin, GameSharedPlugin);

    // run the app
    #[cfg(not(feature = "debug_dump"))]
    apps.run();

    #[cfg(feature = "debug_dump")]
    {
        match &mut apps {
            Apps::Client { app, config } => {
                debug_dump(app, "client");
            }
            Apps::Server { app, config } => {
                debug_dump(app, "server");
            }
            Apps::ClientAndServer {
                client_app,
                client_config,
                server_app,
                server_config,
            } => {
                debug_dump(client_app, "test_client");
                debug_dump(server_app, "test_server");
            }
            Apps::HostServer {
                app,
                client_config,
                server_config,
            } => {
                debug_dump(app, "host_server");
            }
        }
    }
}

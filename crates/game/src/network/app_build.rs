use atom_internal::plugins::AtomClientPlugins;
use bevy::{asset::AssetPlugin, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use log_layers::{file_layer, LogLayersPlugin};
use network::{app::BuildApp, settings::Settings};

pub struct GameBuildApp;

impl BuildApp for GameBuildApp {
    /// An `App` that contains both the client and server plugins
    #[cfg(not(target_family = "wasm"))]
    fn combined_app(settings: Settings) -> App {
        use atom_internal::plugins::AtomHostServerPlugins;

        let mut app = App::new();

        app.add_plugins(LogLayersPlugin);

        LogLayersPlugin::add_layer(
            &mut app,
            file_layer::file_layer_with_filename("combine".to_string()),
        );

        app.add_plugins(AtomHostServerPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // uncomment for unthrottled FPS
                present_mode: bevy::window::PresentMode::AutoNoVsync,
                title: "host_server".to_string(),
                ..default()
            }),
            ..default()
        }));
        if settings.client.inspector {
            app.add_plugins(WorldInspectorPlugin::new());
        }

        app
    }

    /// Build the server app with the `ServerPlugins` added.
    #[cfg(not(target_family = "wasm"))]
    fn server_app(settings: Settings) -> App {
        use atom_internal::plugins::AtomServerPlugins;
        use bevy::log::LogPlugin;
        use bevy::prelude::*;

        let mut app = App::new();
        app.add_plugins(LogLayersPlugin);

        LogLayersPlugin::add_layer(
            &mut app,
            file_layer::file_layer_with_filename("server".to_string()),
        );

        if !settings.server.headless {
            app.add_plugins(AtomServerPlugins.build().disable::<LogPlugin>());
        } else {
            app.add_plugins(AtomServerPlugins);
        }

        // if settings.server.inspector {
        //     app.add_plugins(WorldInspectorPlugin::new());
        // }

        app
    }

    /// Build the client app with the `ClientPlugins` added.
    /// Takes in a `net_config` parameter so that we configure the network transport.
    fn client_app(settings: Settings) -> App {
        let mut app = App::new();

        app.add_plugins(LogLayersPlugin);

        LogLayersPlugin::add_layer(
            &mut app,
            file_layer::file_layer_with_filename("client".to_string()),
        );

        app.add_plugins(
            AtomClientPlugins
                .set(AssetPlugin {
                    // https://github.com/bevyengine/bevy/issues/10157
                    meta_check: bevy::asset::AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        // uncomment for unthrottled FPS
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        title: "client".to_string(),
                        ..default()
                    }),
                    ..default()
                }),
        );
        if settings.client.inspector {
            app.add_plugins(WorldInspectorPlugin::new());
        }
        app
    }
}

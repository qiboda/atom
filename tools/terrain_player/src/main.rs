use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use clap::Parser;

use bevy::prelude::*;

use player::PlayerOrders;

mod player;
mod shapes;

#[derive(Parser, Debug, Resource)]
#[command(author, version, about, long_about=None)]
struct Args {
    #[arg(short, long)]
    filename: PathBuf,
}

fn main() {
    let args = Args::parse();
    println!("filename: {:?}", args.filename);

    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .insert_resource(args)
        .insert_resource(PlayerOrders::default())
        .add_systems(Startup, startup);

    app.run();
}

// 单一光标。
// 启动和清除不同的线程。

fn startup(args: Res<Args>, mut player_order: ResMut<PlayerOrders>) {
    if let Ok(file) = File::open(args.filename.clone()) {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            if let Ok(order) = serde_json::from_str(line.as_str()) {
                player_order.push_order(order);
            }
        }
    }
}

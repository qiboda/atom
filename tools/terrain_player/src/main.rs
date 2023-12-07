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
        let mut line_num = 0;
        for line in reader.lines().flatten() {
            match serde_json::from_str(line.as_str()) {
                Ok(order) => {
                    player_order.push_order(order);
                    line_num += 1;
                }
                Err(err) => {
                    error!("parse line error: {}, err: {}", line, err);
                }
            }
        }
        info!("file line num: {}", line_num);
        info!("first line info: {:?}", player_order.get_order(0).unwrap());
    } else {
        error!("parse file name can not open: {:?}", args.filename);
    }
}

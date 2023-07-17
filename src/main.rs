mod args;
mod config;
mod controller;
mod draw;
mod input;
mod model;
mod tmux;
mod tmux_context;
mod daemon;
use std::error::Error;

use args::parse_config_from_args;
use controller::create_controller;
use input::input_loop;
use model::{State, create_process};
use tmux_context::create_tmux_context;

#[macro_use]
extern crate log;

use log::info;

fn main() -> Result<(), Box<dyn Error>> {
    let file = std::fs::File::create("/tmp/proctmux.log").unwrap();
    env_logger::builder()
        .target(env_logger::Target::Pipe(Box::new(file)))
        .filter_level(log::LevelFilter::Trace)
        .init();
    let config = parse_config_from_args()?;
    let tmux_context = create_tmux_context("proctmux background processes".to_string())?;
    info!("Starting proctmux");

    let state = State {
        current_selection: 0,
        processes: vec![
            create_process(1, "Simple Echo", "echo hi"),
            create_process(
                2,
                "Echo x10",
                "for i in `seq 1 10`; do echo $i; sleep 2 ; done",
            ),
            create_process(3, "vim", "vim"),
        ],
        messages: vec![]
    };

    input_loop(create_controller(config, state, tmux_context)?)?;
    Ok(())
}

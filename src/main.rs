mod args;
mod config;
mod controller;
mod draw;
mod input;
mod model;
mod tmux;
mod tmux_context;
mod tmux_daemon;
mod daemon;

use std::error::Error;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};

use log::info;

use args::parse_config_from_args;
use controller::Controller;
use input::input_loop;
use model::{Process, State};
use tmux_context::TmuxContext;

#[macro_use]
extern crate log;

fn main() -> Result<(), Box<dyn Error>> {
    let file = std::fs::File::create("/tmp/proctmux.log").unwrap();
    env_logger::builder()
        .target(env_logger::Target::Pipe(Box::new(file)))
        .filter_level(log::LevelFilter::Trace)
        .init();

    let config = parse_config_from_args()?;

    let tmux_context = TmuxContext::new(
        &config.general.detatched_session_name,
        config.general.kill_existing_session
    )?;

    info!("Starting proctmux");

    let state = State::new(
        vec![
            Process::new(1, "Simple Echo", "echo hi"),
            Process::new(
                2,
                "Echo x10",
                "for i in `seq 1 3`; do echo $i; sleep 1 ; done",
            ),
            Process::new(3, "vim", "vim"),
        ]
    );

    let (tx, rx) = channel();

    std::thread::spawn(move || {
        for line in rx {
            info!("channel: {}", line);
        }
    });

    tmux_daemon::watch_for_dead_panes(&tmux_context.session, tx.clone())?;
    tmux_daemon::watch_for_dead_panes(&tmux_context.detached_session, tx)?;

    let controller = Arc::new(Mutex::new(Controller::new(config, state, tmux_context)?));
    input_loop(controller.clone())?;

    Ok(())
}

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
use daemon::receive_dead_pids;
use input::input_loop;
use model::{State};
use tmux_context::TmuxContext;
use tmux_daemon::TmuxDaemon;

#[macro_use]
extern crate log;

fn main() -> Result<(), Box<dyn Error>> {
    let config = parse_config_from_args()?;

    let file = std::fs::File::create(config.log_file.clone()).unwrap();
    env_logger::builder()
        .target(env_logger::Target::Pipe(Box::new(file)))
        .filter_level(log::LevelFilter::Trace)
        .init();

    info!("Starting proctmux");

    let tmux_context = TmuxContext::new(
        &config.general.detached_session_name,
        config.general.kill_existing_session
    )?;

    let state = State::new(&config);

    let (sender, receiver) = channel();

    let mut tmux_daemon_attached = TmuxDaemon::new(&tmux_context.session_id)?;
    tmux_daemon_attached.listen_for_dead_panes(sender.clone())?;
    let mut tmux_daemon_detached = TmuxDaemon::new(&tmux_context.detached_session_id)?;
    tmux_daemon_detached.listen_for_dead_panes(sender)?;

    let controller = Arc::new(Mutex::new(Controller::new(state, tmux_context)?));
    controller.lock().unwrap().on_startup()?;

    receive_dead_pids(receiver, controller.clone());
    input_loop(controller.clone())?;

    tmux_daemon_attached.kill()?;
    tmux_daemon_detached.kill()?;
    controller.lock().unwrap().on_exit()?;

    Ok(())
}

mod config;
mod model;
mod draw;
mod tmux;
mod tmux_context;
mod event;
mod args;
use std::io::{stdout, Write};

use args::parse_config_from_args;
use draw::draw_screen;
use event::event_loop;
use model::{create_command, State};
use termion::{clear, cursor, raw::IntoRawMode};

use tmux_context::create_tmux_context;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = parse_config_from_args()?;

    let tmux_context = create_tmux_context("proctmux detached panes".to_string())?;

    let state = State {
        current_selection: 0,
        commands: vec![
            create_command(1, "Simple Echo", "echo hi"),
            create_command(
                2,
                "Echo x10",
                "for i in 1 2 3 4 5 6 7 8 9 10 ; do echo $i; sleep 2 ; done",
            ),
            create_command(3, "vim", "vim"),
        ],
        messages: vec![],
        tmux_context,
    };

    let mut stdout = stdout().into_raw_mode()?;

    write!(stdout, "{}", cursor::Hide)?;

    draw_screen(&state, &stdout)?;
    event_loop(state, &stdout, &config)?;

    write!(
        stdout,
        "{}{}{}",
        cursor::Goto(0, 1),
        clear::All,
        cursor::Show
    )?;

    Ok(())
}


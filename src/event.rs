use crate::config::ProcTmuxConfig;
use crate::{draw::draw_screen, model::State};
use std::io::{stdin, Stdout, Write};
use termion::{event::Key, input::TermRead};

fn matches_key(key: Key, acceptable_keys: &[String]) -> bool {
    match key {
        Key::Char(c) => acceptable_keys.contains(&c.to_string()),
        _ => false,
    }
}

pub fn event_loop(
    mut state: State,
    mut stdout: &Stdout,
    config: &ProcTmuxConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    state.tmux_context.prepare()?;

    let stdin = stdin();
    let keybinding = &config.keybinding;

    for c in stdin.keys() {
        match c {
            Ok(key) => {
                if matches_key(key, &keybinding.quit) {
                    state.tmux_context.cleanup()?;
                    break;
                } else if matches_key(key, &keybinding.down) {
                    state.next_command();
                    draw_screen(&state, stdout)?;
                } else if matches_key(key, &keybinding.up) {
                    state.previous_command();
                    draw_screen(&state, stdout)?;
                } else if matches_key(key, &keybinding.start) {
                    state.start_process();
                    draw_screen(&state, stdout)?;
                } else if matches_key(key, &keybinding.stop) {
                    state.halt_process();
                    draw_screen(&state, stdout)?;
                }
            },
            Err(e) => {
                write!(stdout, "{}", e)?;
            }
        }
    }

    Ok(())
}

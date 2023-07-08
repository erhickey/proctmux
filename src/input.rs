use std::error::Error;
use std::io::stdin;

use termion::{event::Key, input::TermRead};

use crate::controller::Controller;

fn matches_key(key: Key, acceptable_keys: &[String]) -> bool {
    match key {
        Key::Char(c) => acceptable_keys.contains(&c.to_string()),
        _ => false,
    }
}

pub fn input_loop(mut controller: Controller) -> Result<(), Box<dyn Error>> {
    let stdin = stdin();
    let keybinding = &controller.config.keybinding.clone();

    for c in stdin.keys() {
        match c {
            Ok(key) => {
                if matches_key(key, &keybinding.quit) {
                    controller.on_keypress_quit()?;
                    controller.on_exit()?;
                    break;
                } else if matches_key(key, &keybinding.down) {
                    controller.on_keypress_down()?;
                } else if matches_key(key, &keybinding.up) {
                    controller.on_keypress_up()?;
                } else if matches_key(key, &keybinding.start) {
                    controller.on_keypress_start()?;
                } else if matches_key(key, &keybinding.stop) {
                    controller.on_keypress_stop()?;
                }
            },
            Err(e) => {
                controller.on_error(Box::new(e));
            }
        }
    }

    Ok(())
}

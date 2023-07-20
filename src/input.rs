use std::error::Error;
use std::io::stdin;
use std::sync::{Arc, Mutex};

use termion::{event::Key, input::TermRead};

use crate::config::KeybindingConfig;
use crate::controller::Controller;

pub fn input_loop(controller: Arc<Mutex<Controller>>) -> Result<(), Box<dyn Error>> {
    let keybinding = controller.lock().unwrap().config.keybinding.clone();
    let stdin = stdin();

    for c in stdin.keys() {
        info!("Got keypress: {:?}", c);
        if let Ok(result) = handle_normal_mode_keypresses(controller.clone(), c, &keybinding) {
            if result {
                break;
            }
        }
    }

    Ok(())
}

fn handle_normal_mode_keypresses(
    controller: Arc<Mutex<Controller>>,
    pressed_key: Result<Key, std::io::Error>,
    keybinding: &KeybindingConfig,
) -> Result<bool, Box<dyn Error>> {
    match pressed_key {
        Ok(key) => {
            if matches_key(key, &keybinding.quit) {
                controller.lock().unwrap().on_keypress_quit()?;
                return Ok(true);
            } else if matches_key(key, &keybinding.down) {
                controller.lock().unwrap().on_keypress_down()?;
            } else if matches_key(key, &keybinding.up) {
                controller.lock().unwrap().on_keypress_up()?;
            } else if matches_key(key, &keybinding.start) {
                controller.lock().unwrap().on_keypress_start()?;
            } else if matches_key(key, &keybinding.stop) {
                controller.lock().unwrap().on_keypress_stop()?;
            }
        }
        Err(e) => {
            controller.lock().unwrap().on_error(Box::new(e));
        }
    }
    Ok(false)
}

fn matches_key(key: Key, acceptable_keys: &[String]) -> bool {
    match key {
        Key::Char(c) => acceptable_keys.contains(&c.to_string()),
        _ => false,
    }
}

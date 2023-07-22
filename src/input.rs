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
        if controller.lock().unwrap().is_entering_filter_text() {
            handle_filter_entry_keypresses(controller.clone(), c, &keybinding)?;
        } else if handle_normal_mode_keypresses(controller.clone(), c, &keybinding).unwrap_or(false) {
            break;
        }
    }

    Ok(())
}

fn handle_filter_entry_keypresses(
    controller: Arc<Mutex<Controller>>,
    pressed_key: Result<Key, std::io::Error>,
    keybinding: &KeybindingConfig,
) -> Result<(), Box<dyn Error>> {
    match pressed_key {
        Ok(key) => {
            if matches_key(key, &keybinding.filter_submit) {
                controller.lock().unwrap().on_filter_done()?;
                info!("filter done");
            } else if matches_key(key, &keybinding.filter) {
                controller.lock().unwrap().on_filter_set(None)?;
                info!("cancelled filter");
                controller.lock().unwrap().on_filter_done()?;
                info!("filter done");
            } else if key == Key::Backspace {
                let filter_text = controller.lock().unwrap().get_filter_text();
                if let Some(mut filter_text) = filter_text {
                    filter_text.pop();
                    info!("setting filter text: {}", filter_text);
                    controller.lock().unwrap().on_filter_set(Some(filter_text))?;
                }
            } else if let Key::Char(c) = key {
                let filter_text = controller.lock().unwrap().get_filter_text();
                let mut new_filter_text = match filter_text {
                    Some(filter_text) => filter_text,
                    None => String::new(),
                };
                new_filter_text.push(c);
                info!("setting filter text: {:?}", new_filter_text);
                controller.lock().unwrap().on_filter_set(Some(new_filter_text))?;
            }
        }
        Err(e) => {
            controller.lock().unwrap().on_error(Box::new(e));
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
            } else if matches_key(key, &keybinding.filter) {
                controller.lock().unwrap().on_filter_start()?;
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

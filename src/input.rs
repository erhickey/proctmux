use std::error::Error;
use std::io::stdin;
use std::sync::{Arc, Mutex};

use termion::{event::Key, input::TermRead};

use crate::config::KeybindingConfig;
use crate::controller::Controller;

pub fn input_loop(
    controller: Arc<Mutex<Controller>>,
    keybinding: KeybindingConfig,
) -> Result<(), Box<dyn Error>> {
    let stdin = stdin();

    for c in stdin.keys() {
        trace!("Got keypress: {:?}", c);
        if controller.lock().unwrap().is_entering_filter_text() {
            handle_filter_entry_keypresses(controller.clone(), c, &keybinding)?;
        } else if handle_normal_mode_keypresses(controller.clone(), c, &keybinding).unwrap_or(false)
        {
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
            if keybinding.filter_submit.contains(&key) {
                controller.lock().unwrap().on_filter_done()?;
                trace!("filter done");
            } else if keybinding.filter.contains(&key) {
                controller.lock().unwrap().on_filter_set(None)?;
                trace!("cancelled filter");
                controller.lock().unwrap().on_filter_done()?;
                trace!("filter done");
            } else if key == Key::Backspace {
                let filter_text = controller.lock().unwrap().filter_text();
                if let Some(mut filter_text) = filter_text {
                    filter_text.pop();
                    info!("setting filter text: {}", filter_text);
                    controller
                        .lock()
                        .unwrap()
                        .on_filter_set(Some(filter_text))?;
                }
            } else if let Key::Char(c) = key {
                let filter_text = controller.lock().unwrap().filter_text();
                let mut new_filter_text = match filter_text {
                    Some(filter_text) => filter_text,
                    None => String::new(),
                };
                new_filter_text.push(c);
                info!("setting filter text: {:?}", new_filter_text);
                controller
                    .lock()
                    .unwrap()
                    .on_filter_set(Some(new_filter_text))?;
            }
        }
        Err(e) => {
            controller.lock().unwrap().on_error(Box::new(e))?;
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
            if keybinding.quit.contains(&key) {
                controller.lock().unwrap().on_keypress_quit()?;
                return Ok(true);
            } else if keybinding.down.contains(&key) {
                controller.lock().unwrap().on_keypress_down()?;
            } else if keybinding.up.contains(&key) {
                controller.lock().unwrap().on_keypress_up()?;
            } else if keybinding.start.contains(&key) {
                controller.lock().unwrap().on_keypress_start()?;
            } else if keybinding.stop.contains(&key) {
                controller.lock().unwrap().on_keypress_stop()?;
            } else if keybinding.filter.contains(&key) {
                controller.lock().unwrap().on_filter_start()?;
            } else if keybinding.switch_focus.contains(&key) {
                controller.lock().unwrap().on_keypress_switch_focus()?;
            }
        }
        Err(e) => {
            controller.lock().unwrap().on_error(Box::new(e))?;
        }
    }
    Ok(false)
}

use std::error::Error;
use std::io::stdin;
use std::sync::{Arc, Mutex};

use termion::{event::Key, input::TermRead};

use crate::config::KeybindingConfig;
use crate::controller::Controller;

pub fn input_loop(controller: Arc<Mutex<Controller>>, keybinding: KeybindingConfig) {
    let stdin = stdin();

    for c in stdin.keys() {
        trace!("Got keypress: {:?}", c);
        match c {
            Ok(key) => {
                if controller.lock().unwrap().is_entering_filter_text() {
                    if let Err(e) =
                        handle_filter_entry_keypresses(controller.clone(), key, &keybinding)
                    {
                        error!("Error handling filter keypress {:?}: {}", key, e);
                    }
                } else {
                    if let Err(e) =
                        handle_normal_mode_keypresses(controller.clone(), key, &keybinding)
                    {
                        error!("Error handling keypress {:?}: {}", key, e);
                        // TODO: remove this break, see TODO below
                        break;
                    }
                }
            }
            Err(e) => {
                error!("Error reading stdin: {}", e);
            }
        }
    }
}

fn handle_filter_entry_keypresses(
    controller: Arc<Mutex<Controller>>,
    key: Key,
    keybinding: &KeybindingConfig,
) -> Result<(), Box<dyn Error>> {
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
    Ok(())
}
fn handle_normal_mode_keypresses(
    controller: Arc<Mutex<Controller>>,
    key: Key,
    keybinding: &KeybindingConfig,
) -> Result<(), Box<dyn Error>> {
    if keybinding.quit.contains(&key) {
        controller.lock().unwrap().on_keypress_quit()?;
        // Throwing an error to break the stdin loop and close stdin
        // TODO: probably use async_stdin and the booleancondvar to
        // watch for when the stdin loop should be broken
        return Err(Box::new(core::fmt::Error {}));
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
    Ok(())
}

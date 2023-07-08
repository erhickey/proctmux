use std::error::Error;
use std::io::stdin;
use std::sync::{Arc, Mutex};
use std::thread::spawn;

use termion::{event::Key, input::TermRead};

use crate::controller::Controller;

fn matches_key(key: Key, acceptable_keys: &[String]) -> bool {
    match key {
        Key::Char(c) => acceptable_keys.contains(&c.to_string()),
        _ => false,
    }
}

pub fn input_loop(controller: Controller) -> Result<(), Box<dyn Error>> {
    let keybinding = controller.config.keybinding.clone();

    let a_controller = Arc::new(Mutex::new(controller));
    a_controller.lock().unwrap().on_startup()?;

    let stdin = stdin();

    for c in stdin.keys() {
        match c {
            Ok(key) => {
                if matches_key(key, &keybinding.quit) {
                    a_controller.lock().unwrap().on_keypress_quit()?;
                    break;
                } else if matches_key(key, &keybinding.down) {
                    a_controller.lock().unwrap().on_keypress_down()?;
                } else if matches_key(key, &keybinding.up) {
                    a_controller.lock().unwrap().on_keypress_up()?;
                } else if matches_key(key, &keybinding.start) {
                    if let Some((pid, process_index)) = a_controller.lock().unwrap().on_keypress_start()? {
                        let cont = a_controller.clone();
                        spawn(move || unsafe {
                            let mut file = std::fs::File::create("foo.txt").unwrap();
                            use std::io::prelude::*;
                            let l1 = format!("{}\n", pid);
                            let _ = file.write_all(l1.as_bytes());
                            // BUG: waitpid returns immediately, should options be something other than 0?
                            libc::waitpid(pid, std::ptr::null_mut(), 0);
                            let _ = file.write_all(b"waitpid done");
                            let _ = cont.lock().unwrap().on_process_terminated(process_index);
                        });
                    }
                } else if matches_key(key, &keybinding.stop) {
                    a_controller.lock().unwrap().on_keypress_stop()?;
                }
            },
            Err(e) => {
                a_controller.lock().unwrap().on_error(Box::new(e));
            }
        }
    }

    a_controller.lock().unwrap().on_exit()?;
    Ok(())
}

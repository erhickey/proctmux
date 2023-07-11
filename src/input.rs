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

    let am_controller = Arc::new(Mutex::new(controller));
    am_controller.lock().unwrap().on_startup()?;

    let stdin = stdin();

    for c in stdin.keys() {
        match c {
            Ok(key) => {
                if matches_key(key, &keybinding.quit) {
                    am_controller.lock().unwrap().on_keypress_quit()?;
                    break;
                } else if matches_key(key, &keybinding.down) {
                    am_controller.lock().unwrap().on_keypress_down()?;
                } else if matches_key(key, &keybinding.up) {
                    am_controller.lock().unwrap().on_keypress_up()?;
                } else if matches_key(key, &keybinding.start) {
                    if let Some((pid, process_index)) = am_controller.lock().unwrap().on_keypress_start()? {
                        watch_pid(am_controller.clone(), pid, process_index);
                    }
                } else if matches_key(key, &keybinding.stop) {
                    am_controller.lock().unwrap().on_keypress_stop()?;
                }
            },
            Err(e) => {
                am_controller.lock().unwrap().on_error(Box::new(e));
            }
        }
    }

    am_controller.lock().unwrap().on_exit()?;
    Ok(())
}

fn watch_pid(controller: Arc<Mutex<Controller>>, pid: i32, process_index: usize) {
    spawn(move || unsafe {
        let mut file = std::fs::File::create("foo.txt").unwrap();
        use std::io::prelude::*;
        let l1 = format!("{}\n", pid);
        let _ = file.write_all(l1.as_bytes());
        // BUG: waitpid returns immediately, should options be something other than 0?
        libc::waitpid(pid, std::ptr::null_mut(), 0);
        let _ = file.write_all(b"waitpid done");
        let _ = controller.lock().unwrap().on_process_terminated(process_index);
    });
}

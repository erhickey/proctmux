use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::process::ChildStdin;
use std::sync::mpsc::Sender;
use std::thread::{sleep, spawn};
use std::time;

use crate::tmux;

pub fn watch_for_dead_panes(session: &str, sender: Sender<String>) -> Result<(), Box<dyn Error>> {
    let command_mode = tmux::command_mode(session).unwrap();
    subscribe_to_pane_dead_notifications(command_mode.stdin.unwrap())?;

    spawn(move || {
        let mut buf_reader = BufReader::new(command_mode.stdout.unwrap());
        loop {
            let mut buf = String::new();
            match buf_reader.read_line(&mut buf) {
                Ok(_) => {
                    if buf == "" {
                        sender.send("shit".to_string()).unwrap();
                        sleep(time::Duration::from_millis(100));
                    } else {
                        sender.send(buf).unwrap();
                    }
                },
                _ => return
            }
        }
    });

    Ok(())
}

fn subscribe_to_pane_dead_notifications(mut input: ChildStdin) -> std::io::Result<()> {
    let cmd = "refresh-client -B pane_dead_notification:%*:\"#{pane_dead} #S:#I.#P #{pane_pid}\"";
    input.write_all(cmd.as_bytes())
}

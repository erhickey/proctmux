use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout};
use std::sync::mpsc::Sender;
use std::thread::{sleep, spawn};
use std::time;

use crate::tmux;

pub struct TmuxDaemon {
    process: Child,
    stdout: Option<ChildStdout>,
    stdin: ChildStdin,
}

impl TmuxDaemon {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut process = tmux::command_mode()?;
        let stdin = process.stdin.take().unwrap();
        let stdout = process.stdout.take();

        Ok(TmuxDaemon {
            process,
            stdout,
            stdin
        })
    }

    fn subscribe_to_pane_dead_notifications(&mut self) -> std::io::Result<()> {
        let cmd = "refresh-client -B pane_dead_notification:%*:\"#{pane_dead} #S:#I.#P #{pane_pid}\"";
        self.stdin.write_all(cmd.as_bytes())
    }

    pub fn kill(&mut self) -> std::io::Result<()> {
        self.stdin.write_all(b"kill-session")?;
        self.process.kill()
    }

    pub fn listen_for_dead_panes(&mut self, sender: Sender<String>) -> Result<(), Box<dyn Error>> {
        self.subscribe_to_pane_dead_notifications()?;
        let mut buf_reader = BufReader::new(self.stdout.take().unwrap());

        spawn(move || {
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
}

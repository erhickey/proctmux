use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, ExitStatus};
use std::sync::mpsc::Sender;
use std::thread::spawn;

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
        let cmd = "refresh-client -B pane_dead_notification:%*:\"#{pane_dead} #S:#I.#P #{pane_pid}\"\n";
        self.stdin.write_all(cmd.as_bytes())
    }

    pub fn kill(&mut self) -> std::io::Result<ExitStatus> {
        self.stdin.write_all(b"kill-session\n")?;
        self.process.wait()  // make sure stdin is closed
    }

    pub fn listen_for_dead_panes(&mut self, sender: Sender<String>) -> Result<(), Box<dyn Error>> {
        self.subscribe_to_pane_dead_notifications()?;
        let mut buf_reader = BufReader::new(self.stdout.take().unwrap());

        spawn(move || {
            loop {
                let mut buf = String::new();
                match buf_reader.read_line(&mut buf) {
                    Ok(_) => {
                        sender.send(buf).unwrap();
                    },
                    _ => return
                }
            }
        });

        Ok(())
    }
}

use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, ExitStatus};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::thread::spawn;

use crate::tmux;

pub struct TmuxDaemon {
    target: String,
    process: Child,
    stdout: Option<ChildStdout>,
    stdin: ChildStdin,
    running: Arc<AtomicBool>,
}

impl TmuxDaemon {
    pub fn new(target: &str) -> Result<Self, Box<dyn Error>> {
        info!("Starting tmux command mode (Target {}) process", target);
        let mut process = tmux::command_mode(target)?;
        let stdin = process.stdin.take().unwrap();
        let stdout = process.stdout.take();

        Ok(TmuxDaemon {
            target: target.to_string(),
            process,
            stdout,
            stdin,
            running: Arc::new(AtomicBool::new(true))
        })
    }

    fn subscribe_to_pane_dead_notifications(&mut self) -> std::io::Result<()> {
        info!("Subscribing to pane dead notifications (Target: {})", self.target);
        let cmd = format!(
            "refresh-client -B pane_dead_notification_{}:%*:\"#{{pane_dead}} #{{pane_pid}}\"\n",
            clean(&self.target)
        );
        self.stdin.write_all(cmd.as_bytes())
    }

    pub fn kill(&mut self) -> std::io::Result<ExitStatus> {
        info!("Killing tmux command mode (Target: {}) process", self.target);
        self.running.store(false, Ordering::Relaxed);
        self.process.kill()?;
        self.process.wait()  // make sure stdin is closed
    }

    pub fn listen_for_dead_panes(&mut self, sender: Sender<i32>) -> Result<(), Box<dyn Error>> {
        self.subscribe_to_pane_dead_notifications()?;
        let mut buf_reader = BufReader::new(self.stdout.take().unwrap());
        let running = self.running.clone();

        spawn(move || {
            while running.load(Ordering::Relaxed) {
                let mut buf = String::new();
                match buf_reader.read_line(&mut buf) {
                    Ok(_) => {
                        if let Some(pid) = parse_pane_dead_notification(buf) {
                            sender.send(pid).unwrap();
                        }
                    },
                    _ => return
                }
            }
        });

        Ok(())
    }
}

fn parse_pane_dead_notification(line: String) -> Option<i32> {
    if line.starts_with("%subscription-changed pane_dead_notification_") {
        let ss: Vec<&str> = line.split(' ').collect();
        if ss[ss.len() - 2] == "1" {
            return ss[ss.len() - 1].trim().parse().ok();
        }
    }
    None
}

fn clean(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).collect()
}

use std::{process::{Command, Output}, io::Error};

const TMUX_SESSION_NAME: &str = "proctmux detached panes";

pub fn start_detached_session() -> Result<Output, Error> {
    Command::new("tmux")
            .arg("new-session")
            .arg("-d")
            .arg("-s")
            .arg(TMUX_SESSION_NAME)
            .output()
}

pub fn set_remain_on_exit(on: bool) -> Result<Output, Error> {
     Command::new("tmux")
             .arg("set-option")
             .arg("-t")
             .arg("0:0")
             .arg("remain-on-exit")
             .arg(if on { "on" } else { "off" })
             .output()
}

pub fn stop_detached_session() -> Result<Output, Error> {
    Command::new("tmux")
            .arg("kill-session")
            .arg("-t")
            .arg(TMUX_SESSION_NAME)
            .output()
}

pub fn break_pane(window_id: usize, window_label: &str) -> Result<Output, Error> {
    Command::new("tmux")
            .arg("break-pane")
            .arg("-d")
            .arg("-s")
            .arg("0:0.1")
            .arg("-t")
            .arg(format!("{}:{}", TMUX_SESSION_NAME, window_id))
            .arg("-n")
            .arg(window_label)
            .output()
}

pub fn join_pane(window_id: usize) -> Result<Output, Error> {
    Command::new("tmux")
            .arg("join-pane")
            .arg("-d")
            .arg("-h")
            .arg("-l")
            .arg("70%")
            .arg("-s")
            .arg(format!("{}:{}", TMUX_SESSION_NAME, window_id))
            .arg("-t")
            .arg("0:0")
            .output()
}

pub fn create_pane(command: &str) -> Result<Output, Error> {
    Command::new("tmux")
            .arg("split-window")
            .arg("-d")
            .arg("-h")
            .arg("-l")
            .arg("70%")
            .arg("-t")
            .arg("0:0.0")
            .arg(command)
            .output()
}

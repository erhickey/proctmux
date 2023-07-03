use std::{process::{Command, Output}, io::Error};

pub fn current_session() -> Result<Output, Error> {
    Command::new("tmux")
            .arg("display-message")
            .arg("-p")
            .arg("#S")
            .output()
}

pub fn current_window() -> Result<Output, Error> {
    Command::new("tmux")
            .arg("display-message")
            .arg("-p")
            .arg("#I")
            .output()
}

pub fn current_pane() -> Result<Output, Error> {
    Command::new("tmux")
            .arg("display-message")
            .arg("-p")
            .arg("#P")
            .output()
}

pub fn start_detached_session(session: &str) -> Result<Output, Error> {
    Command::new("tmux")
            .arg("new-session")
            .arg("-d")
            .arg("-s")
            .arg(session)
            .output()
}

pub fn set_remain_on_exit(session: &str, window: usize, on: bool) -> Result<Output, Error> {
     Command::new("tmux")
             .arg("set-option")
             .arg("-t")
             .arg(format!("{}:{}", session, window))
             .arg("remain-on-exit")
             .arg(if on { "on" } else { "off" })
             .output()
}

pub fn kill_session(session: &str) -> Result<Output, Error> {
    Command::new("tmux")
            .arg("kill-session")
            .arg("-t")
            .arg(session)
            .output()
}

pub fn break_pane(
    source_session: &str,
    source_window: usize,
    source_pane: usize,
    dest_session: &str,
    dest_window: usize,
    window_label: &str
) -> Result<Output, Error> {
    Command::new("tmux")
            .arg("break-pane")
            .arg("-d")
            .arg("-s")
            .arg(format!("{}:{}.{}", source_session, source_window, source_pane))
            .arg("-t")
            .arg(format!("{}:{}", dest_session, dest_window))
            .arg("-n")
            .arg(window_label)
            .output()
}

pub fn join_pane(
    source_session: &str,
    source_window: usize,
    dest_session: &str,
    dest_window: usize,
    dest_pane: usize
) -> Result<Output, Error> {
    Command::new("tmux")
            .arg("join-pane")
            .arg("-d")
            .arg("-h")
            .arg("-l")
            .arg("70%")
            .arg("-s")
            .arg(format!("{}:{}", source_session, source_window))
            .arg("-t")
            .arg(format!("{}:{}.{}", dest_session, dest_window, dest_pane))
            .output()
}

pub fn create_pane(session: &str, window: usize, pane: usize, command: &str) -> Result<Output, Error> {
    Command::new("tmux")
            .arg("split-window")
            .arg("-d")
            .arg("-h")
            .arg("-l")
            .arg("70%")
            .arg("-t")
            .arg(format!("{}:{}.{}", session, window, pane))
            .arg("-P")
            .arg("-F")
            .arg("#{pane_index}")
            .arg(command)
            .output()
}

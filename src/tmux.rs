use std::error::Error;
use std::io::Result as IoResult;
use std::process::{Child, Command, Stdio, Output};

fn clean_output(s: &str) -> String {
    s.replace("\n", "")
}

pub fn read_bytes(output: IoResult<Output>) -> Result<String, Box<dyn Error>> {
    Ok(clean_output(&String::from_utf8(output?.stdout)?))
}

pub fn list_sessions() -> IoResult<Output> {
    Command::new("tmux")
            .arg("list-sessions")
            .arg("-F")
            .arg("#{session_name}")
            .output()
}
pub fn current_session() -> IoResult<Output> {
    Command::new("tmux")
            .arg("display-message")
            .arg("-p")
            .arg("#S")
            .output()
}

pub fn current_window() -> IoResult<Output> {
    Command::new("tmux")
            .arg("display-message")
            .arg("-p")
            .arg("#I")
            .output()
}

pub fn current_pane() -> IoResult<Output> {
    Command::new("tmux")
            .arg("display-message")
            .arg("-p")
            .arg("#P")
            .output()
}

pub fn start_detached_session(session: &str) -> IoResult<Output> {
    Command::new("tmux")
            .arg("new-session")
            .arg("-d")
            .arg("-s")
            .arg(session)
            .output()
}

pub fn set_remain_on_exit(session: &str, window: usize, on: bool) -> IoResult<Output> {
     Command::new("tmux")
             .arg("set-option")
             .arg("-t")
             .arg(format!("{}:{}", session, window))
             .arg("remain-on-exit")
             .arg(if on { "on" } else { "off" })
             .output()
}

pub fn kill_session(session: &str) -> IoResult<Output> {
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
) -> IoResult<Output> {
    Command::new("tmux")
            .arg("break-pane")
            .arg("-d")
            .arg("-s")
            .arg(format!("{}:{}.{}", source_session, source_window, source_pane))
            .arg("-t")
            .arg(format!("{}:{}", dest_session, dest_window))
            .arg("-n")
            .arg(window_label)
            .arg("-P")
            .arg("-F")
            .arg("#{pane_index}")
            .output()
}

pub fn join_pane(
    source_session: &str,
    source_window: usize,
    dest_session: &str,
    dest_window: usize,
    dest_pane: usize
) -> IoResult<Output> {
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

pub fn kill_pane(session: &str, window: usize, pane: usize) -> IoResult<Output> {
    Command::new("tmux")
            .arg("kill-pane")
            .arg("-t")
            .arg(format!("{}:{}.{}", session, window, pane))
            .output()
}

pub fn create_pane(session: &str, window: usize, pane: usize, command: &str) -> IoResult<Output> {
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

pub fn get_pane_pid(session: &str, window: usize, pane: usize) -> IoResult<Output> {
    Command::new("tmux")
        .arg("display-message")
        .arg("-p")
        .arg("-t")
        .arg(format!("{}:{}.{}", session, window, pane))
        .arg("#{pane_pid}")
        .output()
}

pub fn command_mode(session: &str) -> IoResult<Child> {
    Command::new("tmux")
        .arg("-C")
        .arg("attach-session")
        .arg("-t")
        .arg(session)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
}

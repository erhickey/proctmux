use std::error::Error;
use std::io::Result as IoResult;
use std::process::{Child, Command, Output, Stdio};

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
        .arg("#{session_id}")
        .output()
}

pub fn current_pane() -> IoResult<Output> {
    Command::new("tmux")
        .arg("display-message")
        .arg("-p")
        .arg("#{pane_id}")
        .output()
}

pub fn start_detached_session(session_name: &str) -> IoResult<Output> {
    Command::new("tmux")
        .arg("new-session")
        .arg("-d")
        .arg("-s")
        .arg(session_name)
        .arg("-P")
        .arg("-F")
        .arg("#{session_id}")
        .output()
}

pub fn set_remain_on_exit(pane_id: &str, on: bool) -> IoResult<Output> {
    Command::new("tmux")
        .arg("set-option")
        .arg("-t")
        .arg(pane_id)
        .arg("remain-on-exit")
        .arg(if on { "on" } else { "off" })
        .output()
}

pub fn kill_session(session_id: &str) -> IoResult<Output> {
    Command::new("tmux")
        .arg("kill-session")
        .arg("-t")
        .arg(session_id)
        .output()
}

pub fn break_pane(
    pane_id: &str,
    dest_session: &str,
    dest_window: usize,
    window_label: &str,
) -> IoResult<Output> {
    Command::new("tmux")
        .arg("break-pane")
        .arg("-d")
        .arg("-s")
        .arg(pane_id)
        .arg("-t")
        .arg(format!("{}:{}", dest_session, dest_window))
        .arg("-n")
        .arg(window_label)
        .output()
}

pub fn join_pane(target_pane: &str, dest_pane: &str) -> IoResult<Output> {
    Command::new("tmux")
        .arg("join-pane")
        .arg("-d")
        .arg("-h")
        .arg("-l")
        .arg("70%")
        .arg("-s")
        .arg(target_pane)
        .arg("-t")
        .arg(dest_pane)
        .output()
}

pub fn kill_pane(pane_id: &str) -> IoResult<Output> {
    Command::new("tmux")
        .arg("kill-pane")
        .arg("-t")
        .arg(pane_id)
        .output()
}

pub fn create_pane(pane_id: &str, command: &str) -> IoResult<Output> {
    Command::new("tmux")
        .arg("split-window")
        .arg("-d")
        .arg("-h")
        .arg("-l")
        .arg("70%")
        .arg("-t")
        .arg(pane_id)
        .arg("-P")
        .arg("-F")
        .arg("#{pane_id}")
        .arg(command)
        .output()
}

pub fn create_detached_pane(
    dest_session: &str,
    dest_window: usize,
    window_label: &str,
    command: &str,
) -> IoResult<Output> {
    Command::new("tmux")
        .arg("new-window")
        .arg("-d")
        .arg("-t")
        .arg(format!("{}:{}", dest_session, dest_window))
        .arg("-n")
        .arg(window_label)
        .arg("-P")
        .arg("-F")
        .arg("#{pane_id}")
        .arg(command)
        .output()
}

pub fn get_pane_pid(pane_id: &str) -> IoResult<Output> {
    Command::new("tmux")
        .arg("display-message")
        .arg("-p")
        .arg("-t")
        .arg(pane_id)
        .arg("#{pane_pid}")
        .output()
}

pub fn select_pane(pane_id: &str) -> IoResult<Output> {
    Command::new("tmux")
        .arg("select-pane")
        .arg("-t")
        .arg(pane_id)
        .output()
}

pub fn control_mode(session_id: &str) -> IoResult<Child> {
    Command::new("tmux")
        .arg("-C")
        .arg("attach-session")
        .arg("-t")
        .arg(session_id)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
}

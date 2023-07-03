use std::io::Error;
use std::process::Output;

use crate::tmux;

pub struct TmuxContext {
    detached_session: String,
    session: String,
    window: usize,
    pane: usize
}

pub fn create_tmux_context(detached_session: String) -> Result<TmuxContext, Error> {
    let session = match String::from_utf8(tmux::current_session()?.stdout) {
        Ok(val) => val.replace("\n", ""),
        Err(err) => panic!("Error: Could not retrieve tmux session id: {}", err)
    };
    let window = match String::from_utf8(tmux::current_window()?.stdout) {
        Ok(val) => val.replace("\n", ""),
        Err(err) => panic!("Error: Could not retrieve tmux window id: {}", err)
    };
    let pane = match String::from_utf8(tmux::current_pane()?.stdout) {
        Ok(val) => val.replace("\n", ""),
        Err(err) => panic!("Error: Could not retrieve tmux pane id: {}", err)
    };

    let window_id = match window.parse() {
        Ok(i) => i,
        Err(err) => panic!("Error: Failed to parse tmux window {}: {}", window, err)
    };
    let pane_id = match pane.parse() {
        Ok(i) => i,
        Err(err) => panic!("Error: Failed to parse tmux pane {}: {}", pane, err)
    };

    Ok(TmuxContext {
        detached_session,
        session,
        window: window_id,
        pane: pane_id,
    })
}

impl TmuxContext {
    pub fn prepare(&self) -> Result<Output, Error> {
        tmux::start_detached_session(&self.detached_session)?;
        tmux::set_remain_on_exit(&self.session, self.window, true)
    }

    pub fn cleanup(&self) -> Result<Output, Error> {
        tmux::kill_session(&self.detached_session)?;
        tmux::set_remain_on_exit(&self.session, self.window, false)
    }

    pub fn break_pane(&self, source_pane: usize, dest_window: usize, window_label: &str) -> Result<Output, Error> {
        tmux::break_pane(
            &self.session,
            self.window,
            source_pane,
            &self.detached_session,
            dest_window,
            window_label)
    }

    pub fn join_pane(&self, target_window: usize) -> Result<Output, Error> {
        tmux::join_pane(
            &self.detached_session,
            target_window,
            &self.session,
            self.window
        )
    }

    pub fn create_pane(&self, command: &str) -> Result<Output, Error> {
        tmux::create_pane(&self.session, self.window, self.pane, command)
    }
}

use std::collections::HashSet;
use std::error::Error;
use std::io::Result as IoResult;
use std::process::Output;

use log::info;

use crate::tmux;

pub struct TmuxContext {
    pane_id: String,
    pub session_id: String,
    pub detached_session_id: String,
}

impl TmuxContext {
    pub fn new(detached_session: &str, kill_existing_session: bool) -> Result<Self, Box<dyn Error>> {
        let pane_id = match tmux::read_bytes(tmux::current_pane()) {
            Ok(val) => val,
            Err(e) => panic!("Error: Could not retrieve tmux pane id: {}", e),
        };
        let session_id = match tmux::read_bytes(tmux::current_session()) {
            Ok(val) => val,
            Err(e) => panic!("Error: Could not retrieve tmux session id: {}", e),
        };

        let existing_session_names: HashSet<String> = tmux::read_bytes(tmux::list_sessions())?
            .split("\n")
            .map(|s| s.to_string())
            .collect();

        let detached_session_id = match {
            if existing_session_names.contains(detached_session){
                if kill_existing_session {
                    info!("Killing existing session: {}", detached_session);
                    tmux::kill_session(detached_session)?;
                    tmux::read_bytes(tmux::start_detached_session(detached_session))
                } else {
                    panic!("Session '{}' already exists", detached_session);
                }
            } else {
                tmux::read_bytes(tmux::start_detached_session(detached_session))
            }
        } {
            Ok(val) => val,
            Err(e) => panic!("Error: Could not retrieve tmux detached session id: {}", e)
        };

        info!(
            "creating tmux context: pane_id: {}, session: {}, detached_session: {}",
            pane_id,
            session_id,
            detached_session_id,
        );

        Ok(TmuxContext {
            pane_id,
            session_id,
            detached_session_id,
        })
    }

    pub fn prepare(&self) -> IoResult<Output> {
        tmux::set_remain_on_exit(&self.pane_id, true)
    }

    pub fn cleanup(&self) -> IoResult<Output> {
        let output = tmux::kill_session(&self.detached_session_id);
        tmux::set_remain_on_exit(&self.pane_id, false)?;
        output
    }

    pub fn break_pane(
        &self,
        pane_id: &str,
        dest_window: usize,
        window_label: &str,
    ) -> IoResult<Output> {
        info!(
            "breaking pane: pane_id: {}, dest_window: {}, window_label: {}",
            pane_id,
            dest_window,
            window_label
        );
        let output = tmux::break_pane(pane_id, &self.detached_session_id, dest_window, window_label);
        tmux::set_remain_on_exit(pane_id, true)?;
        output
    }

    pub fn join_pane(&self, pane_id: &str) -> IoResult<Output> {
        info!("Joining pane_id: {} to pane_id: {}", pane_id, self.pane_id);
        tmux::join_pane(pane_id, &self.pane_id)
    }

    pub fn create_pane(&self, command: &str) -> Result<String, Box<dyn Error>> {
        info!("Creating pane: {}", command);
        tmux::read_bytes(tmux::create_pane(&self.pane_id, command))
    }

    pub fn get_pane_pid(&self, pane_id: &str) -> Result<i32, Box<dyn Error>> {
        Ok(tmux::read_bytes(tmux::get_pane_pid(pane_id))?.parse()?)
    }
}

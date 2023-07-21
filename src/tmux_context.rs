use std::collections::HashSet;
use std::error::Error;
use std::io::Result as IoResult;
use std::process::Output;

use log::info;

use crate::model::{TmuxAddress, TmuxAddressChange};
use crate::tmux;

pub struct TmuxContext {
    pub detached_session: String,
    pub session: String,
    window: usize,
    pane: usize,
}

impl TmuxContext {
    pub fn new(detached_session: &str, kill_existing_session: bool) -> Result<Self, Box<dyn Error>> {
        let session = match tmux::read_bytes(tmux::current_session()) {
            Ok(val) => val,
            Err(e) => panic!("Error: Could not retrieve tmux session id: {}", e),
        };
        let window_id = match tmux::read_bytes(tmux::current_window())?.parse() {
            Ok(val) => val,
            Err(e) => panic!("Error: Could not retrieve tmux window id: {}", e),
        };
        let pane_id = match tmux::read_bytes(tmux::current_pane())?.parse() {
            Ok(val) => val,
            Err(e) => panic!("Error: Could not retrieve tmux pane id: {}", e),
        };

        let existing_session_names: HashSet<String> = tmux::read_bytes(tmux::list_sessions())?
            .split("\n")
            .map(|s| s.to_string())
            .collect();

        if existing_session_names.contains(detached_session){
            if kill_existing_session {
                info!("Killing existing session: {}", detached_session);
                tmux::kill_session(detached_session)?;
                tmux::start_detached_session(detached_session)?;
            } else {
                return Err(format!("Session '{}' already exists", detached_session).into());
            }
        } else {
            tmux::start_detached_session(detached_session)?;
        }

        info!(
            "creating tmux context: session: {}, detached_session: {}, window_id: {}, pane_id: {}",
            session,
            detached_session,
            window_id,
            pane_id
        );

        Ok(TmuxContext {
            detached_session: detached_session.to_string(),
            session,
            window: window_id,
            pane: pane_id,
        })
    }

    pub fn prepare(&self) -> IoResult<Output> {
        tmux::set_remain_on_exit(&self.session, self.window, true)
    }

    pub fn cleanup(&self) -> IoResult<Output> {
        tmux::kill_session(&self.detached_session)?;
        tmux::set_remain_on_exit(&self.session, self.window, false)
    }

    pub fn break_pane(
        &self,
        source_pane: usize,
        dest_window: usize,
        window_label: &str,
    ) -> Result<TmuxAddressChange, Box<dyn Error>> {
        info!(
            "breaking pane: source_pane: {}, dest_window: {}, window_label: {}",
            source_pane,
            dest_window,
            window_label
        );

        let pane_id = tmux::read_bytes(tmux::break_pane(
            &self.session,
            self.window,
            source_pane,
            &self.detached_session,
            dest_window,
            window_label)
        )?.parse().unwrap_or(0);

        tmux::set_remain_on_exit(&self.detached_session, dest_window, true)?;

        Ok(TmuxAddressChange {
            new_address: TmuxAddress::new(&self.detached_session, dest_window, Some(pane_id)),
            old_address: TmuxAddress::new(&self.session, self.window, Some(source_pane)),
        })
    }

    pub fn join_pane(&self, target_window: usize) -> IoResult<TmuxAddress> {
        info!("Joining window: {} to session: {}", target_window, self.session);

        tmux::join_pane(
            &self.detached_session,
            target_window,
            &self.session,
            self.window,
            self.pane
        )?;

        Ok(TmuxAddress::new(&self.session, self.window, Some(self.pane + 1)))
    }

    pub fn kill_pane(&self, pane: usize) -> IoResult<Output> {
        tmux::kill_pane(&self.session, self.window, pane)
    }

    pub fn create_pane(&self, command: &str) -> Result<TmuxAddress, Box<dyn Error>> {
        info!("Creating pane: {}", command);
        let pane_id = tmux::read_bytes(tmux::create_pane(&self.session, self.window, self.pane, command))?.parse()?;
        info!("Pane created: {}", pane_id);
        Ok(TmuxAddress::new(&self.session, self.window, Some(pane_id)))
    }

    pub fn get_pane_pid(&self, pane: usize) -> Result<i32, Box<dyn Error>> {
        Ok(tmux::read_bytes(tmux::get_pane_pid(&self.session, self.window, pane))?.parse()?)
    }
}

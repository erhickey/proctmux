use std::error::Error;
use std::process::Output;

use crate::model::{TmuxAddress, TmuxAddressChange};
use crate::tmux;

use log::info;
pub struct TmuxContext {
    detached_session: String,
    session: String,
    window: usize,
    pane: usize
}

pub fn create_tmux_context(detached_session: String) -> Result<TmuxContext, Box<dyn Error>> {
    let session = match String::from_utf8(tmux::current_session()?.stdout) {
        Ok(val) => clean_output(&val),
        Err(e) => panic!("Error: Could not retrieve tmux session id: {}", e),
    };
    let window = match String::from_utf8(tmux::current_window()?.stdout) {
        Ok(val) => clean_output(&val),
        Err(e) => panic!("Error: Could not retrieve tmux window id: {}", e),
    };
    let pane = match String::from_utf8(tmux::current_pane()?.stdout) {
        Ok(val) => clean_output(&val),
        Err(e) => panic!("Error: Could not retrieve tmux pane id: {}", e),
    };

    let window_id = parse_id(&window)?;

    let pane_id = parse_id(&pane)?;
    info!("creating tmux context: session: {}, detached_session: {}, window_id: {}, pane_id: {}", session, detached_session, window_id, pane_id);

    Ok(TmuxContext {
        detached_session,
        session,
        window: window_id,
        pane: pane_id,
    })
}

fn clean_output(s: &str) -> String {
    s.replace("\n", "")
}

fn parse_id(pane_or_window: &str) -> Result<usize, Box<dyn Error>> {
    let pane_or_window = clean_output(pane_or_window);
    let id: usize = pane_or_window.parse()?;
    Ok(id)
}
fn parse_pid(pid: &str) -> Result<i32, Box<dyn Error>> {
    let pid = clean_output(pid);
    let id: i32= pid.parse()?;
    Ok(id)
}

impl TmuxContext {
    pub fn prepare(&self) -> Result<Output, Box<dyn Error>> {
        tmux::start_detached_session(&self.detached_session)?;
        let output = tmux::set_remain_on_exit(&self.session, self.window, true)?;
        Ok(output)
    }

    pub fn cleanup(&self) -> Result<Output, Box<dyn Error>> {
        tmux::kill_session(&self.detached_session)?;
        let output = tmux::set_remain_on_exit(&self.session, self.window, false)?;
        Ok(output)
    }

    pub fn break_pane(
        &self,
        source_pane: usize,
        dest_window: usize,
        window_label: &str,
    ) -> Result<TmuxAddressChange, Box<dyn Error>> {
        info!("breaking pane: source_pane: {}, dest_window: {}, window_label: {}", source_pane, dest_window, window_label);
        tmux::break_pane(
            &self.session,
            self.window,
            source_pane,
            &self.detached_session,
            dest_window,
            window_label)?;
        tmux::set_remain_on_exit(&self.detached_session, dest_window, true)?;

        let address_change = TmuxAddressChange {
            new_address: TmuxAddress::new(&self.detached_session, dest_window, None),
            old_address: TmuxAddress::new(&self.session, self.window, Some(source_pane)),
        };
        Ok(address_change)
    }

    pub fn join_pane(&self, target_window: usize) -> Result<TmuxAddressChange, Box<dyn Error>> {
        tmux::join_pane(
            &self.detached_session,
            target_window,
            &self.session,
            self.window,
            self.pane, 
        )?;

        let address_change = TmuxAddressChange {
            new_address: TmuxAddress::new(&self.session, target_window, Some(self.pane + 1)),
            old_address: TmuxAddress::new(&self.detached_session, self.window, None),
        };
        info!("Joining pane_id: {} to session: {}", 
            address_change.new_address.pane_id.unwrap(), 
            address_change.new_address.session_name);
        Ok(address_change)

    }

    pub fn kill_pane(&self, pane: usize) -> Result<Output, Box<dyn Error>> {
        let output =  tmux::kill_pane(&self.session, self.window, pane)?;
        Ok(output)
    }

    pub fn create_pane(&self, command: &str) -> Result<usize, Box<dyn Error>> {
        let pane = tmux::create_pane(&self.session, self.window, self.pane, command)?;
        let pane_id = parse_id(&String::from_utf8(pane.stdout)?)?;
        info!("creating pane: {}", pane_id);
        Ok(pane_id)
    }

    pub fn get_pane_pid(&self, pane: usize) -> Result<i32, Box<dyn Error>> {
        let pid = tmux::get_pane_pid(&self.session, self.window, pane)?;
        let pid = parse_pid(&String::from_utf8(pid.stdout)?)?;
        Ok(pid)
    }
}

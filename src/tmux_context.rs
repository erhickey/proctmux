use std::collections::HashSet;
use std::error::Error;
use std::io::Write;
use std::process::{ChildStderr, ChildStdin, ChildStdout, Output};

use crate::model::{TmuxAddress, TmuxAddressChange};
use crate::tmux;

use log::info;
pub struct TmuxContext {
    detached_session: String,
    session: String,
    window: usize,
    picker_pane: usize,
    active_proc_pane: usize,
    command_mode_attached: TmuxCommandModeStreams,
    command_mode_detached: TmuxCommandModeStreams,
}

pub struct TmuxCommandModeStreams {
    input: ChildStdin,
    out: ChildStdout,
    err: ChildStderr,
}

pub fn create_tmux_context(
    detached_session: &str, 
    kill_existing_session: bool 
   ) -> Result<TmuxContext, Box<dyn Error>> {
    let existing_session_names = String::from_utf8(tmux::list_sessions()?.stdout)?;
    let existing_session_names: HashSet<_> = HashSet::from_iter(existing_session_names.split("\n"));

    if existing_session_names.contains(detached_session){
        if kill_existing_session {
            info!("Killing existing session: {}", detached_session);
            tmux::kill_session(detached_session)?;
        } else {
            return Err(format!("Session '{}' already exists", detached_session).into());
        }
    }

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
    info!(
        "creating tmux context: session: {}, detached_session: {}, window_id: {}, pane_id: {}",
        session,
        detached_session,
        window_id,
        pane_id
    );

    let (a_in, a_out, a_err) = tmux::command_mode_streams(&session)?;
    let (d_in, d_out, d_err) = tmux::command_mode_streams(&detached_session)?;

    Ok(TmuxContext {
        detached_session: detached_session.to_string(),
        session,
        window: window_id,
        picker_pane: pane_id,
        active_proc_pane: pane_id + 1,
        command_mode_attached: TmuxCommandModeStreams {
            input: a_in,
            out: a_out,
            err: a_err
        },
        command_mode_detached: TmuxCommandModeStreams {
            input: d_in,
            out: d_out,
            err: d_err
        },
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
        Ok(tmux::set_remain_on_exit(&self.session, self.window, true)?)
    }

    pub fn cleanup(&self) -> Result<Output, Box<dyn Error>> {
        tmux::kill_session(&self.detached_session)?;
        Ok(tmux::set_remain_on_exit(&self.session, self.window, false)?)
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

        tmux::break_pane(
            &self.session,
            self.window,
            source_pane,
            &self.detached_session,
            dest_window,
            window_label)?;
        tmux::set_remain_on_exit(&self.detached_session, dest_window, true)?;

        Ok(TmuxAddressChange {
            new_address: TmuxAddress::new(&self.detached_session, dest_window, None),
            old_address: TmuxAddress::new(&self.session, self.window, Some(source_pane)),
        })
    }

    pub fn join_pane(&self, target_window: usize) -> Result<TmuxAddressChange, Box<dyn Error>> {
        tmux::join_pane(
            &self.detached_session,
            target_window,
            &self.session,
            self.window,
            self.picker_pane
        )?;

        let address_change = TmuxAddressChange {
            new_address: TmuxAddress::new(&self.session, target_window, Some(self.active_proc_pane)),
            old_address: TmuxAddress::new(&self.detached_session, self.window, None),
        };

        info!(
            "Joining pane_id: {} to session: {}",
            address_change.new_address.pane_id.unwrap(),
            address_change.new_address.session_name
        );

        Ok(address_change)
    }

    pub fn kill_pane(&self, pane: usize) -> Result<Output, Box<dyn Error>> {
        Ok(tmux::kill_pane(&self.session, self.window, pane)?)
    }

    pub fn create_pane(&self, command: &str) -> Result<TmuxAddress, Box<dyn Error>> {
        let pane = tmux::create_pane(&self.session, self.window, self.picker_pane, command)?;
        let pane_id = parse_id(&String::from_utf8(pane.stdout)?)?;
        info!("creating pane: {}", pane_id);
        Ok(TmuxAddress::new(&self.session, self.window, Some(self.active_proc_pane)))
    }

    pub fn get_pane_pid(&self, pane: usize) -> Result<i32, Box<dyn Error>> {
        let pid = tmux::get_pane_pid(&self.session, self.window, pane)?;
        Ok(parse_pid(&String::from_utf8(pid.stdout)?)?)
    }

    pub fn subscribe_to_pane_dead_notifications(&mut self) -> std::io::Result<()> {
        let cmd = "refresh-client -B pane_dead_notification:%*:\"#{pane_dead} #S:#I.#P #{pane_pid}\"";
        self.command_mode_detached.input.write_all(cmd.as_bytes())?;
        self.command_mode_attached.input.write_all(cmd.as_bytes())
    }
}

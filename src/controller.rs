use std::error::Error;
use std::io::Stdout;

use termion:: raw::RawTerminal;

use crate::config::ProcTmuxConfig;
use crate::draw::{draw_screen, init_screen, prepare_screen_for_exit};
use crate::model::{PaneStatus, ProcessStatus, State, StateMutation};
use crate::tmux_context::TmuxContext;
use log::info;

pub struct Controller {
    pub config: ProcTmuxConfig,
    state: State,
    tmux_context: TmuxContext,
    stdout: RawTerminal<Stdout>
}

pub fn create_controller(
    config: ProcTmuxConfig,
    state: State,
    tmux_context: TmuxContext
) -> Result<Controller, Box<dyn Error>> {
    Ok(Controller {
        config,
        state,
        tmux_context,
        stdout: init_screen()?
    })
}

impl Controller {
    fn draw_screen(&self) -> Result<(), Box<dyn Error>> {
        draw_screen(&self.stdout, &self.state)
    }

    pub fn on_startup(&self) -> Result<(), Box<dyn Error>> {
        self.draw_screen()?;
        self.tmux_context.prepare()?;
        Ok(())
    }

    pub fn on_exit(&self) -> Result<(), Box<dyn Error>> {
        prepare_screen_for_exit(&self.stdout)
    }

    pub fn on_keypress_quit(&self) -> Result<(), Box<dyn Error>> {
        self.state.processes.iter()
            .filter(|process| process.pane_status == PaneStatus::Dead && process.pane_id.is_some())
            .for_each(|process| {
                self.tmux_context.kill_pane(process.pane_id.unwrap()).unwrap();
        });
        self.tmux_context.cleanup()?;
        Ok(())
    }

    pub fn on_keypress_down(&mut self) -> Result<(), Box<dyn Error>> {
        info!("on_keypress_down");
        self.break_pane();
        self.state = StateMutation::on(self.state.clone()).next_process().commit();
        self.join_pane();
        self.draw_screen()
    }

    pub fn on_keypress_up(&mut self) -> Result<(), Box<dyn Error>> {
        info!("on_keypress_up");
        self.break_pane();
        self.state = StateMutation::on(self.state.clone()).previous_process().commit();
        self.join_pane();
        self.draw_screen()
    }

    pub fn on_error(&mut self, err: Box<dyn Error>) {
        self.state.messages.push(format!("{}", err));
    }

    pub fn on_keypress_start(&mut self) -> Result<Option<(i32, usize)>, Box<dyn Error>> {
        let result = self.start_process();
        self.draw_screen()?;
        Ok(result)
    }

    pub fn on_keypress_stop(&mut self) -> Result<(), Box<dyn Error>> {
        self.halt_process();
        self.draw_screen()
    }

    pub fn on_process_terminated(&mut self, process_index: usize) -> Result<(), Box<dyn Error>> {
        self.state = StateMutation::on(self.state.clone())
            .mark_process_status(ProcessStatus::Halted, process_index)
            .mark_pane_status(PaneStatus::Dead, process_index)
            .commit();
        self.draw_screen()
    }

    pub fn break_pane(&mut self) {
        let process = self.state.current_process();
        info!("inner break_pane status {:?} {:?}", process.pane_status, process.pane_id);
        if process.pane_status != PaneStatus::Null && process.pane_id.is_some() {
            let address_change = self.tmux_context
                .break_pane(process.pane_id.unwrap(), process.id, &process.label)
                .unwrap();
            self.state = StateMutation::on(self.state.clone())
                .set_pane_id(address_change.new_address.pane_id)
                .commit();
        }
    }

    pub fn join_pane(&mut self) {
        let process = self.state.current_process();
        if process.pane_status != PaneStatus::Null {
            let address_change = self.tmux_context.join_pane(process.id).unwrap();
            self.state = StateMutation::on(self.state.clone())
                .set_pane_id(address_change.new_address.pane_id)
                .commit();
        }
    }

    pub fn start_process(&mut self) -> Option<(i32, usize)> {
        let process = self.state.current_process().clone();

        if process.status != ProcessStatus::Halted {
            return None;
        }

        let mut state_mutation = StateMutation::on(self.state.clone())
            .mark_current_process_status(ProcessStatus::Running);
            

        if process.pane_status == PaneStatus::Dead && process.pane_id.is_some() {
            self.tmux_context.kill_pane(process.pane_id.unwrap()).unwrap();
            // return None;
        }

        if process.pane_status == PaneStatus::Null  || process.pane_status == PaneStatus::Dead {
            let pane_id = self.tmux_context.create_pane(&process.command).unwrap();
            state_mutation = state_mutation
                .mark_current_pane_status(PaneStatus::Running)
                .set_pane_id(Some(pane_id));
            self.state = state_mutation.commit();
            return Some((
                self.tmux_context.get_pane_pid(pane_id).unwrap(),
                self.state.current_selection
            ));
        }

        None
    }

    pub fn halt_process(&mut self) {
        let process = self.state.current_process();

        if process.status != ProcessStatus::Running {
            return;
        }

        if let Some(pane_id) = process.pane_id {
            let pane_pid = self.tmux_context.get_pane_pid(pane_id).unwrap();
            unsafe { libc::kill(pane_pid, libc::SIGKILL) };
            self.state = StateMutation::on(self.state.clone())
                .mark_current_process_status(ProcessStatus::Halting)
                .commit();
        }
    }
}

use std::collections::HashMap;
use std::error::Error;
use std::io::Stdout;

use log::info;
use termion::raw::RawTerminal;

use crate::config::ProcTmuxConfig;
use crate::draw::{draw_screen, init_screen, prepare_screen_for_exit};
use crate::model::{PaneStatus, ProcessStatus, State, StateMutation, GUIStateMutation, Mutator};
use crate::tmux_context::TmuxContext;

pub struct Controller {
    pub config: ProcTmuxConfig,
    state: State,
    tmux_context: TmuxContext,
    stdout: RawTerminal<Stdout>,
}

impl Controller {
    pub fn new(
        config: ProcTmuxConfig,
        state: State,
        tmux_context: TmuxContext,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Controller {
            config,
            state,
            tmux_context,
            stdout: init_screen()?,
        })
    }

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
        self.state
            .processes
            .iter()
            .filter(|process| process.pane_status == PaneStatus::Dead)
            .for_each(|process| {
                if let Some(addy) = &process.tmux_address {
                    if let Some(pane_id) = addy.pane_id {
                        self.tmux_context.kill_pane(pane_id).unwrap();
                    }
                }
            });
        self.tmux_context.cleanup()?;
        Ok(())
    }

    pub fn on_keypress_down(&mut self) -> Result<(), Box<dyn Error>> {
        info!("on_keypress_down");
        self.break_pane();
        self.state = StateMutation::on(self.state.clone())
            .next_process()
            .commit();
        self.join_pane();
        self.draw_screen()
    }

    pub fn on_keypress_up(&mut self) -> Result<(), Box<dyn Error>> {
        info!("on_keypress_up");
        self.break_pane();
        self.state = StateMutation::on(self.state.clone())
            .previous_process()
            .commit();
        self.join_pane();
        self.draw_screen()
    }

    pub fn on_error(&mut self, err: Box<dyn Error>) {
        let gui_state = GUIStateMutation::on(self.state.gui_state.clone())
            .add_message(format!("{}", err))
            .commit();
        self.state = StateMutation::on(self.state.clone())
            .set_gui_state(gui_state)
            .commit();
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

    pub fn get_processes_to_pid(&self) -> HashMap<usize, Option<i32>> {
        let m: HashMap<_,_>= self.state.processes.iter().map(|process| {
            if process.status == ProcessStatus::Halted {
                return (process.id, None)
            }
            if let Some(addy) = &process.tmux_address {
                if let Some(pane_id) = addy.pane_id {
                    if let Ok(pid) = self.tmux_context.get_pane_pid(pane_id) {
                        return (process.id, Some(pid))
                    }
                } 
            }
            (process.id, None)
        }).collect();
        info!("get_processes_to_pid: {:?}", m);
        m
    }

    pub fn break_pane(&mut self) {
        let process = self.state.current_process();
        if process.pane_status != PaneStatus::Null {
            if let Some(addy) = &process.tmux_address {
                if let Some(pane_id) = addy.pane_id {
                    let address_change = self
                        .tmux_context
                        .break_pane(pane_id, process.id, &process.label)
                        .unwrap();
                    self.state = StateMutation::on(self.state.clone())
                        .set_tmux_address(Some(address_change.new_address))
                        .commit();
                }
            }
        }
    }

    pub fn join_pane(&mut self) {
        let process = self.state.current_process();
        if process.pane_status != PaneStatus::Null {
            let address_change = self.tmux_context.join_pane(process.id).unwrap();
            self.state = StateMutation::on(self.state.clone())
                .set_tmux_address(Some(address_change.new_address))
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

        if process.pane_status == PaneStatus::Dead {
            if let Some(addy) = process.tmux_address {
                if let Some(pane_id) = addy.pane_id {
                    self.tmux_context.kill_pane(pane_id).unwrap();
                }
            }
        }

        if process.pane_status == PaneStatus::Null || process.pane_status == PaneStatus::Dead {
            let addy = self.tmux_context.create_pane(&process.command).unwrap();
            let pane_id = addy.pane_id.unwrap();
            state_mutation = state_mutation
                .mark_current_pane_status(PaneStatus::Running)
                .set_tmux_address(Some(addy));
            self.state = state_mutation.commit();
            return Some((
                self.tmux_context
                    .get_pane_pid(pane_id)
                    .unwrap(),
                self.state.current_selection,
            ));
        }

        None
    }

    pub fn halt_process(&mut self) {
        let process = self.state.current_process();

        if process.status != ProcessStatus::Running {
            return;
        }

        if let Some(addy) = &process.tmux_address {
            if let Some(pane_id) = addy.pane_id {
                let pane_pid = self.tmux_context.get_pane_pid(pane_id).unwrap();
                unsafe { libc::kill(pane_pid, libc::SIGKILL) };
                self.state = StateMutation::on(self.state.clone())
                    .mark_current_process_status(ProcessStatus::Halting)
                    .commit();
            }
        }
    }
}

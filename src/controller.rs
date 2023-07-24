use std::error::Error;
use std::io::Stdout;

use log::info;
use termion::raw::RawTerminal;

use crate::config::ProcTmuxConfig;
use crate::draw::{draw_screen, init_screen, prepare_screen_for_exit};
use crate::model::{GUIStateMutation, Mutator, ProcessStatus, State, StateMutation};
use crate::tmux;
use crate::tmux_context::TmuxContext;

pub struct Controller {
    state: State,
    tmux_context: TmuxContext,
    stdout: RawTerminal<Stdout>,
}

impl Controller {
    pub fn new(
        state: State,
        tmux_context: TmuxContext,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Controller {
            state,
            tmux_context,
            stdout: init_screen()?,
        })
    }

    pub fn get_config(&self) -> &ProcTmuxConfig {
        &self.state.config
    }

    pub fn is_entering_filter_text(&self) -> bool {
        self.state.gui_state.entering_filter_text
    }

    pub fn get_filter_text(&self) -> Option<String> {
        self.state.gui_state.filter_text.clone()
    }

    pub fn on_filter_start(&mut self) -> Result<(), Box<dyn Error>> {
        let gui_state = GUIStateMutation::on(self.state.gui_state.clone())
            .start_entering_filter()
            .commit();
        self.state = StateMutation::on(self.state.clone())
            .set_gui_state(gui_state)
            .commit();
        self.draw_screen()
    }
    pub fn on_filter_done(&mut self) -> Result<(), Box<dyn Error>> {
        let gui_state = GUIStateMutation::on(self.state.gui_state.clone())
            .stop_entering_filter()
            .commit();
        self.state = StateMutation::on(self.state.clone())
            .set_gui_state(gui_state)
            .commit();
        self.draw_screen()
    }

    pub fn on_filter_set(&mut self, new_filter_text: Option<String>) -> Result<(), Box<dyn Error>> {
        let gui_state = GUIStateMutation::on(self.state.gui_state.clone())
            .set_filter_text(new_filter_text)
            .commit();
        self.state = StateMutation::on(self.state.clone())
            .set_gui_state(gui_state)
            .commit();
        self.draw_screen()
    }

    fn draw_screen(&self) -> Result<(), Box<dyn Error>> {
        draw_screen(&self.stdout, &self.state)
    }

    pub fn on_startup(&mut self) -> Result<(), Box<dyn Error>> {
        self.tmux_context.prepare()?;

        let ps: Vec<usize> = self.state
            .processes
            .iter()
            .filter(|p| p.config.autostart)
            .map(|p| p.id)
            .collect();
        for p in ps.iter() {
            self.start_process(*p);
        }

        self.draw_screen()
    }

    pub fn on_exit(&self) -> Result<(), Box<dyn Error>> {
        self.tmux_context.cleanup()?;
        prepare_screen_for_exit(&self.stdout)
    }

    pub fn on_keypress_quit(&self) -> Result<(), Box<dyn Error>> {
        self.state
            .processes
            .iter()
            .filter(|process| process.status == ProcessStatus::Halted)
            .for_each(|process| {
                if let Some(pane_id) = &process.pane_id {
                    let _ = tmux::kill_pane(pane_id);
                }
            });
        Ok(())
    }

    pub fn on_keypress_down(&mut self) -> Result<(), Box<dyn Error>> {
        info!("on_keypress_down");
        self.break_pane(self.state.current_proc_id);
        self.state = StateMutation::on(self.state.clone())
            .next_process()
            .commit();
        self.join_pane(self.state.current_proc_id);
        self.draw_screen()
    }

    pub fn on_keypress_up(&mut self) -> Result<(), Box<dyn Error>> {
        info!("on_keypress_up");
        self.break_pane(self.state.current_proc_id);
        self.state = StateMutation::on(self.state.clone())
            .previous_process()
            .commit();
        self.join_pane(self.state.current_proc_id);
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

    pub fn on_keypress_start(&mut self) -> Result<Option<i32>, Box<dyn Error>> {
        let result = self.start_process(self.state.current_proc_id);
        self.draw_screen()?;
        Ok(result)
    }

    pub fn on_keypress_stop(&mut self) -> Result<(), Box<dyn Error>> {
        self.halt_process(self.state.current_proc_id);
        self.draw_screen()
    }

    pub fn on_keypress_switch_focus(&mut self) -> Result<(), Box<dyn Error>> {
        self.switch_focus()
    }

    pub fn switch_focus(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(pane_id) = self.state.current_process().and_then(|p| p.clone().pane_id) {
            tmux::select_pane(&pane_id)?;
        }
        Ok(())
    }

    // pub fn on_process_terminated(&mut self, process_index: usize) -> Result<(), Box<dyn Error>> {
    //     self.state = StateMutation::on(self.state.clone())
    //         .mark_process_status(ProcessStatus::Halted, process_index)
    //         .mark_pane_status(PaneStatus::Dead, process_index)
    //         .commit();
    //     self.draw_screen()
    // }

    // pub fn get_processes_to_pid(&self) -> HashMap<usize, Option<i32>> {
    //     let m: HashMap<_,_>= self.state.processes.iter().map(|process| {
    //         if process.status == ProcessStatus::Halted {
    //             return (process.id, None)
    //         }
    //         if let Some(addy) = &process.tmux_address {
    //             if let Some(pane_id) = addy.pane_id {
    //                 if let Ok(pid) = self.tmux_context.get_pane_pid(pane_id) {
    //                     return (process.id, Some(pid))
    //                 }
    //             }
    //         }
    //         (process.id, None)
    //     }).collect();
    //     info!("get_processes_to_pid: {:?}", m);
    //     m
    // }

    pub fn on_pid_terminated(&mut self, pid: i32) -> Result<(), Box<dyn Error>> {
        info!("on_pid_terminated: {}", pid);
        if let Some(process) = self.state.processes.iter().find(|p| p.pid == Some(pid)) {
            self.state = StateMutation::on(self.state.clone())
                .set_process_status(ProcessStatus::Halted, process.id)
                .set_process_pid(None, process.id)
                .commit();
            self.draw_screen()?;
        }
        Ok(())
    }

    pub fn break_pane(&mut self, process_id: usize) {
        if let Some(process) = self.state.get_process(process_id) {
            if let Some(pane_id) = &process.pane_id {
                // TODO: log error?
                let _ = self
                    .tmux_context
                    .break_pane(pane_id, process.id, &process.label);
            }
        }
    }

    pub fn join_pane(&mut self, process_id: usize) {
        if let Some(process) = self.state.get_process(process_id) {
            if let Some(pane_id) = &process.pane_id {
                // TODO: log error?
                let _ = self.tmux_context.join_pane(pane_id);
            }
        }
    }

    pub fn start_process(&mut self, process_id: usize) -> Option<i32> {
        let process = match self.state.get_process(process_id) {
            Some(p) => p.clone(),
            None => return None
        };

        if process.status != ProcessStatus::Halted {
            return None;
        }

        let mut state_mutation = StateMutation::on(self.state.clone())
            .set_process_status(ProcessStatus::Running, process.id);

        if let Some(pane_id) = &process.pane_id {
            match tmux::kill_pane(pane_id) {
                Ok(_) => {
                    let sm =
                        StateMutation::on(self.state.clone()).set_process_pane_id(None, process.id);
                    self.state = sm.commit();
                }
                Err(_) => {} // TODO: log error?
            }
        }

        let new_pane = if process_id == self.state.current_proc_id {
            self.tmux_context.create_pane(&process.command())
        } else {
            self.tmux_context.create_detached_pane(process.id, &process.label, &process.command())
        };

        match new_pane {
            Ok(pane_id) => {
                let pid = self.tmux_context.get_pane_pid(&pane_id).ok();
                info!(
                    "Started {} process, pid: {}",
                    process.label,
                    pid.unwrap_or(-1)
                );
                state_mutation = state_mutation
                    .set_process_pane_id(Some(pane_id), process.id)
                    .set_process_pid(pid, process.id);
                self.state = state_mutation.commit();
                pid
            }
            Err(_) => None, // TODO: handle this, maybe just log it?
        }
    }

    pub fn halt_process(&mut self, process_id: usize) {
        if let Some(process) = self.state.get_process(process_id) {
            if process.status != ProcessStatus::Running {
                return;
            }

            if let Some(pid) = process.pid {
                unsafe { libc::kill(pid, libc::SIGKILL) };
                self.state = StateMutation::on(self.state.clone())
                    .set_process_status(ProcessStatus::Halting, process.id)
                    .commit();
            }
        }
    }
}

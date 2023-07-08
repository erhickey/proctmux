use std::error::Error;
use std::io::Stdout;
use std::thread::spawn;

use termion:: raw::RawTerminal;

use crate::config::ProcTmuxConfig;
use crate::draw::{draw_screen, init_screen, prepare_screen_for_exit};
use crate::model::{PaneStatus, ProcessStatus, State};
use crate::tmux_context::TmuxContext;

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
        self.tmux_context.cleanup()?;
        // TODO: clean up dead pane(s)
        Ok(())
    }

    pub fn on_keypress_down(&mut self) -> Result<(), Box<dyn Error>> {
        self.break_pane();
        self.state.next_process();
        self.join_pane();
        self.draw_screen()
    }

    pub fn on_keypress_up(&mut self) -> Result<(), Box<dyn Error>> {
        self.break_pane();
        self.state.previous_process();
        self.join_pane();
        self.draw_screen()
    }

    pub fn on_error(&mut self, err: Box<dyn Error>) {
        self.state.messages.push(format!("{}", err));
    }

    pub fn on_keypress_start(&mut self) -> Result<(), Box<dyn Error>> {
        self.start_process();
        self.draw_screen()
    }

    pub fn on_keypress_stop(&mut self) -> Result<(), Box<dyn Error>> {
        self.halt_process();
        self.draw_screen()
    }

    pub fn break_pane(&mut self) {
        let process = self.state.current_process();
        if process.pane_status != PaneStatus::Null && process.pane_id.is_some() {
            self.tmux_context
                .break_pane(process.pane_id.unwrap(), process.id, &process.label)
                .unwrap();
            self.state.set_pane_id(self.state.current_selection, None);
        }
    }

    pub fn join_pane(&mut self) {
        let process = self.state.current_process();
        if process.pane_status != PaneStatus::Null {
            let pane_id = self.tmux_context.join_pane(process.id).unwrap();
            self.state.set_pane_id(self.state.current_selection, Some(pane_id));
        }
    }

    pub fn start_process(&mut self) {
        let process = self.state.current_process().clone();

        if process.status != ProcessStatus::Halted {
            return;
        }

        self.state.set_process_running(self.state.current_selection);

        if process.pane_status == PaneStatus::Dead {
            // TODO: tmux respawn-window
        }

        if process.pane_status == PaneStatus::Null {
            let pane_id = self.tmux_context.create_pane(&process.command).unwrap();
            self.state.set_pane_id(self.state.current_selection, Some(pane_id));
            self.state.set_pane_running(self.state.current_selection);
            self.spawn_pid_watcher_thread(
                self.tmux_context.get_pane_pid(pane_id).unwrap(),
                self.state.current_selection
            );
        }
    }

    pub fn spawn_pid_watcher_thread(&mut self, pid: i32, process_index: usize) {
        // spawn(move || unsafe {
        //     let mut file = std::fs::File::create("foo.txt").unwrap();
        //     use std::io::prelude::*;
        //     let l1 = format!("{}\n", pid);
        //     file.write_all(l1.as_bytes());
        //     // BUG: waitpid returns immediately, should options be something other than 0?
        //     libc::waitpid(pid, std::ptr::null_mut(), 0);
        //     file.write_all(b"waitpid done");
        //     self.state.set_process_halted(process_index);
        //     self.draw_screen();
        // });
    }

    pub fn halt_process(&mut self) {
        let process = self.state.current_process();

        if process.status != ProcessStatus::Running {
            return;
        }

        if let Some(pane_id) = process.pane_id {
            let pane_pid = self.tmux_context.get_pane_pid(pane_id).unwrap();
            unsafe { libc::kill(pane_pid, libc::SIGKILL) };
            self.state.set_process_halting(self.state.current_selection);
        }
    }
}

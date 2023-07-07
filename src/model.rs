use crate::tmux_context::TmuxContext;

#[derive(Clone)]
pub enum ProcessStatus {
    Running = 1,
    Halting = 2,
    Halted = 3,
}

#[derive(Clone, Eq, PartialEq)]
pub enum PaneStatus {
    Null = 1,
    Running = 2,
    Dead = 3,
}

#[derive(Clone)]
pub struct Command {
    pub id: usize,
    pub label: String,
    pub command: String,
    pub status: ProcessStatus,
    pub pane_status: PaneStatus,
    pub pane_id: Option<usize>,
}

pub fn create_command(id: usize, label: &str, command: &str) -> Command {
    Command {
        id,
        label: label.to_string(),
        command: command.to_string(),
        status: ProcessStatus::Halted,
        pane_status: PaneStatus::Null,
        pane_id: None,
    }
}

pub struct State {
    pub current_selection: usize,
    pub commands: Vec<Command>,
    pub messages: Vec<String>,
    pub tmux_context: TmuxContext,
}

impl State {
    pub fn current_command(&mut self) -> Command {
        self.commands[self.current_selection].clone()
    }

    pub fn break_pane(&mut self) {
        let command = self.current_command();
        if command.pane_status != PaneStatus::Null && command.pane_id.is_some() {
            self.tmux_context
                .break_pane(command.pane_id.unwrap(), command.id, &command.label)
                .unwrap();
            self.commands[self.current_selection].pane_id = None;
        }
    }

    pub fn join_pane(&mut self) {
        let command = self.current_command();
        if command.pane_status != PaneStatus::Null {
            let pane_id = self.tmux_context.join_pane(command.id).unwrap();
            self.commands[self.current_selection].pane_id = Some(pane_id);
        }
    }

    pub fn next_command(&mut self) {
        self.messages = vec![];
        self.break_pane();
        if self.current_selection >= self.commands.len() - 1 {
            self.current_selection = 0;
        } else {
            self.current_selection += 1;
        }
        self.join_pane();
    }

    pub fn previous_command(&mut self) {
        self.messages = vec![];
        self.break_pane();
        if self.current_selection == 0 {
            self.current_selection = self.commands.len() - 1;
        } else {
            self.current_selection -= 1;
        }
        self.join_pane();
    }

    pub fn start_process(&mut self) {
        self.commands[self.current_selection].status = ProcessStatus::Running;
        let command = self.current_command();
        if command.pane_status == PaneStatus::Dead {
            // TODO: tmux respawn-window
        }
        if command.pane_status == PaneStatus::Null {
            self.messages = vec![format!("creating pane: {}", command.command)];
            let pane_id = self.tmux_context.create_pane(&command.command).unwrap();
            self.commands[self.current_selection].pane_id = Some(pane_id);
            self.commands[self.current_selection].pane_status = PaneStatus::Running;
        }
    }

    pub fn halt_process(&mut self) {
        self.commands[self.current_selection].status = ProcessStatus::Halted;
    }

    pub fn set_halting(&mut self) {
        self.commands[self.current_selection].status = ProcessStatus::Halting;
    }
}

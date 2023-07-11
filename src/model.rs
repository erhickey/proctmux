#[derive(Clone, Eq, PartialEq)]
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
pub struct Process {
    pub id: usize,
    pub label: String,
    pub command: String,
    pub status: ProcessStatus,
    pub pane_status: PaneStatus,
    pub pane_id: Option<usize>,
}

pub fn create_process(id: usize, label: &str, command: &str) -> Process {
    Process {
        id,
        label: label.to_string(),
        command: command.to_string(),
        status: ProcessStatus::Halted,
        pane_status: PaneStatus::Null,
        pane_id: None
    }
}

pub struct State {
    pub current_selection: usize,
    pub processes: Vec<Process>,
    pub messages: Vec<String>
}

impl State {
    pub fn current_process(&self) -> &Process {
        &self.processes[self.current_selection]
    }

    pub fn next_process(&mut self) {
        if self.current_selection >= self.processes.len() - 1 {
            self.current_selection = 0;
        } else {
            self.current_selection += 1;
        }
    }

    pub fn previous_process(&mut self) {
        if self.current_selection == 0 {
            self.current_selection = self.processes.len() - 1;
        } else {
            self.current_selection -= 1;
        }
    }

    pub fn set_process_running(&mut self, process_index: usize) {
        self.processes[process_index].status = ProcessStatus::Running;
    }

    pub fn set_process_halting(&mut self, process_index: usize) {
        self.processes[process_index].status = ProcessStatus::Halting;
    }

    pub fn set_process_halted(&mut self, process_index: usize) {
        self.processes[process_index].status = ProcessStatus::Halted;
        self.processes[process_index].pane_status = PaneStatus::Dead;
    }

    pub fn set_pane_running(&mut self, process_index: usize) {
        self.processes[process_index].pane_status = PaneStatus::Running;
    }

    pub fn set_pane_id(&mut self, process_index: usize, pane_id: Option<usize>) {
        self.processes[process_index].pane_id = pane_id;
    }
}

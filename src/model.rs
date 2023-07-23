use crate::config::ProcessConfig;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ProcessStatus {
    Running = 1,
    Halting = 2,
    Halted = 3,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Process {
    pub id: usize,
    pub label: String,
    pub status: ProcessStatus,
    pub pane_id: Option<String>,
    pub pid: Option<i32>,
    pub config: ProcessConfig,
}

impl Process {
    pub fn new(id: usize, label: &str, config: ProcessConfig) -> Self {
        Process {
            id,
            label: label.to_string(),
            status: ProcessStatus::Halted,
            pane_id: None,
            pid: None,
            config,
        }
    }

    pub fn command(&self) -> String {
        self.config.shell.clone().unwrap_or(
            self.config.cmd.clone().unwrap_or(vec![])
                .into_iter()
                .map(|s| format!("'{}' ", s))
                .collect()
        )
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct GUIState {
    pub messages: Vec<String>,
    pub filter_text: Option<String>,
    pub entering_filter_text: bool,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct State {
    pub current_selection: usize,
    pub processes: Vec<Process>,
    pub messages: Vec<String>,
    pub gui_state: GUIState,
}

impl State {
    pub fn new(processes: Vec<Process>) -> Self {
        State {
            current_selection: 0,
            processes,
            messages: vec![],
            gui_state: GUIState {
                messages: vec![],
                filter_text: None,
                entering_filter_text: false,
            },
        }
    }

    pub fn current_process(&self) -> &Process {
        &self.processes[self.current_selection]
    }

    pub fn get_filtered_processes(&self) -> Vec<&Process> {
        self.processes
            .iter()
            .filter(|c| {
            if let Some(filter_text) = &self.gui_state.filter_text {
                return c.label.to_lowercase().contains(&filter_text.to_lowercase());
            } 
            true
        }).collect::<Vec<_>>()
    }
}

pub trait Mutator<T> {
    fn on(state: T) -> Self;
    fn commit(self) -> T;
}

pub struct StateMutation {
    init_state: State
}

pub struct GUIStateMutation {
    init_state: GUIState
}

impl Mutator<GUIState> for GUIStateMutation {
    fn on(state: GUIState) -> Self {
        GUIStateMutation{
            init_state: state,
        }
    }

    fn commit(self) -> GUIState {
        self.init_state
    }
}

impl GUIStateMutation {
    pub fn set_filter_text(mut self, text: Option<String>) -> Self {
        self.init_state.filter_text = text;
        self
    }

    pub fn start_entering_filter(mut self) -> Self {
        self.init_state.entering_filter_text = true;
        self
    }

    pub fn stop_entering_filter(mut self) -> Self {
        self.init_state.entering_filter_text = false;
        self
    }

    pub fn add_message(mut self, message: String) -> Self {
        self.init_state.messages.push(message);
        self
    }

    pub fn clear_messages(mut self) -> Self {
        self.init_state.messages.clear();
        self
    }
}

impl Mutator<State> for StateMutation {
    fn on(state: State) -> Self {
        StateMutation{
            init_state: state,
        }
    }

    fn commit(self) -> State {
        self.init_state
    }
}

impl StateMutation {
    pub fn next_process(mut self) -> Self {
        if self.init_state.current_selection >= self.init_state.processes.len() - 1 {
            self.init_state.current_selection = 0;
        } else {
            self.init_state.current_selection += 1;
        }
        self
    }

    pub fn previous_process(mut self) -> Self {
        if self.init_state.current_selection == 0 {
            self.init_state.current_selection = self.init_state.processes.len() - 1;
        } else {
            self.init_state.current_selection -= 1;
        }
        self
    }

    pub fn set_process_status(mut self, status: ProcessStatus, process_id: usize) -> Self {
        self.init_state.processes = self.init_state.processes
            .iter()
            .map(|p| {
                let mut p = p.clone();
                if p.id == process_id {
                    p.status= status.clone();
                }
                p
            })
            .collect();
        self
    }

    pub fn set_process_pane_id(mut self, pane_id: Option<String>, process_id: usize) -> Self {
        self.init_state.processes = self.init_state.processes
            .iter()
            .map(|p| {
                let mut p = p.clone();
                if p.id == process_id {
                    p.pane_id = pane_id.clone();
                }
                p
            })
            .collect();
        self
    }

    pub fn set_process_pid(mut self, pid: Option<i32>, process_id: usize) -> Self {
        self.init_state.processes = self.init_state.processes
            .iter()
            .map(|p| {
                let mut p = p.clone();
                if p.id == process_id {
                    p.pid = pid;
                }
                p
            })
            .collect();
        self
    }

    pub fn set_gui_state(mut self, gui_state: GUIState) -> Self {
        self.init_state.gui_state = gui_state;
        self
    }
}

use std::collections::HashSet;

use crate::config::{ProcTmuxConfig, ProcessConfig};

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
            self.config
                .cmd
                .clone()
                .unwrap_or(vec![])
                .into_iter()
                .map(|s| format!("'{}' ", s))
                .collect(),
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
    pub config: ProcTmuxConfig,
    pub current_proc_id: usize,
    pub processes: Vec<Process>,
    pub gui_state: GUIState,
}

impl State {
    pub fn new(config: &ProcTmuxConfig) -> Self {
        State {
            current_proc_id: 0,
            processes: config
                .procs
                .iter()
                .enumerate()
                .map(|(ix, (k, v))| Process::new(ix + 1, k, v.clone()))
                .collect(),
            config: config.clone(),
            gui_state: GUIState {
                messages: vec![],
                filter_text: None,
                entering_filter_text: false,
            },
        }
    }

    pub fn get_process(&self, process_id: usize) -> Option<&Process> {
        self.processes
            .iter()
            .find(|proc| proc.id == process_id)
    }

    pub fn current_process(&self) -> Option<&Process> {
        self.get_process(self.current_proc_id)
    }

    pub fn get_filtered_processes(&self) -> Vec<&Process> {
        fn filter_by_category(filter_text: &str, proc: &Process) -> bool {
            proc.config
                .categories
                .as_ref()
                .unwrap_or(&vec![])
                .contains(&filter_text.to_lowercase())
        }
        fn filter_by_name_or_meta_tags(filter_text: &str, proc: &Process) -> bool {
            let metas: HashSet<_> = HashSet::from_iter(
                proc.config
                    .meta_tags
                    .as_ref()
                    .unwrap_or(&vec![])
                    .iter()
                    .map(|s| s.to_lowercase()),
            );
            proc.label
                .to_lowercase()
                .contains(&filter_text.to_lowercase())
                || metas.contains(&filter_text.to_lowercase())
        }
        self.processes
            .iter()
            .filter(|proc| {
                if let Some(filter_text) = &self.gui_state.filter_text {
                    let prefix = &self.config.layout.category_search_prefix;
                    if filter_text.starts_with(prefix) {
                        return filter_by_category(&filter_text[prefix.len()..], proc);
                    }
                    return filter_by_name_or_meta_tags(filter_text, proc);
                }
                true
            })
            .collect::<Vec<_>>()
    }
}

pub trait Mutator<T> {
    fn on(state: T) -> Self;
    fn commit(self) -> T;
}

pub struct StateMutation {
    init_state: State,
}

pub struct GUIStateMutation {
    init_state: GUIState,
}

impl Mutator<GUIState> for GUIStateMutation {
    fn on(state: GUIState) -> Self {
        GUIStateMutation { init_state: state }
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
        StateMutation { init_state: state }
    }

    fn commit(self) -> State {
        self.init_state
    }
}

impl StateMutation {
    fn select_first_process(mut self) -> Self {
        let filtered_procs = self.init_state.get_filtered_processes();
        if let Some(first) = filtered_procs.first() {
            self.init_state.current_proc_id = first.id;
        }
        self
    }

    fn move_process_selection(mut self, direction: i8) -> Self {
        let filtered_procs = self.init_state.get_filtered_processes();
        if filtered_procs.is_empty() {
            return self;
        }
        if filtered_procs.len() < 2 {
            return self.select_first_process();
        }
        let available_proc_ids = filtered_procs.iter().map(|p| p.id).collect::<Vec<_>>();
        let current_idx = available_proc_ids
            .iter()
            .position(|&p| p == self.init_state.current_proc_id);
        if current_idx.is_none() {
            return self.select_first_process();
        }
        let current_idx = current_idx.unwrap();
        let new_idx = (current_idx as i32 + direction as i32) % filtered_procs.len() as i32;
        if new_idx < 0 {
            self.init_state.current_proc_id = available_proc_ids[filtered_procs.len() - 1];
        } else {
            self.init_state.current_proc_id = available_proc_ids[new_idx as usize];
        }
        self
    }

    pub fn next_process(self) -> Self {
        self.move_process_selection(1)
    }

    pub fn previous_process(self) -> Self {
        self.move_process_selection(-1)
    }

    pub fn set_process_status(mut self, status: ProcessStatus, process_id: usize) -> Self {
        self.init_state.processes = self
            .init_state
            .processes
            .iter()
            .map(|p| {
                let mut p = p.clone();
                if p.id == process_id {
                    p.status = status.clone();
                }
                p
            })
            .collect();
        self
    }

    pub fn set_process_pane_id(mut self, pane_id: Option<String>, process_id: usize) -> Self {
        self.init_state.processes = self
            .init_state
            .processes
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
        self.init_state.processes = self
            .init_state
            .processes
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

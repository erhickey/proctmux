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

pub struct TmuxAddress {
    pub session_name: String,
    pub window: usize,
    pub pane_id: usize,
}
pub struct TmuxAddressChange {
    pub old_address: TmuxAddress,
    pub new_address: TmuxAddress,
}

impl TmuxAddressChange {
    pub fn new(old_address: TmuxAddress, new_address: TmuxAddress) -> Self {
        TmuxAddressChange {
            old_address,
            new_address
        }
    }
}

impl TmuxAddress {
    pub fn new(session_name: &str, 
        window: usize, 
        pane_id: usize) -> Self {
        TmuxAddress {
            session_name: session_name.to_string(),
            window,
            pane_id
        }
    }
    
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

#[derive(Clone)]
pub struct State {
    pub current_selection: usize,
    pub processes: Vec<Process>,
    pub messages: Vec<String>
}

impl State {

}

impl State {
    pub fn current_process(&self) -> &Process {
        &self.processes[self.current_selection]
    }
}

pub struct StateMutation {
    init_state: State,
}
impl StateMutation{
    pub fn on(state: State) -> Self {
        StateMutation{
            init_state: state,
        }
    }

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

    pub fn mark_current_process_status(mut self, status: ProcessStatus) -> Self {
        self.init_state.processes[self.init_state.current_selection].status = status;
        self
    }   

    pub fn mark_current_pane_status(mut self, status: PaneStatus) -> Self {
        self.init_state.processes[self.init_state.current_selection].pane_status = status;
        self
    }

    pub fn mark_process_status(mut self, status: ProcessStatus, idx: usize) -> Self {
        self.init_state.processes[idx].status = status;
        self
    }   

    pub fn mark_pane_status(mut self, status: PaneStatus, idx: usize) -> Self {
        self.init_state.processes[idx].pane_status = status;
        self
    }


    pub fn set_pane_id(mut self, pane_id: Option<usize>) -> Self {
        self.init_state.processes[self.init_state.current_selection].pane_id = pane_id;
        self
    }

    pub fn commit(self) -> State {
        self.init_state
    }
}




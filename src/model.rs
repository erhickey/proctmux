#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ProcessStatus {
    Running = 1,
    Halting = 2,
    Halted = 3,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum PaneStatus {
    Null = 1,
    Running = 2,
    Dead = 3,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TmuxAddress {
    pub session_name: String,
    pub window: usize,
    pub pane_id: Option<usize>,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TmuxAddressChange {
    pub old_address: TmuxAddress,
    pub new_address: TmuxAddress,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Process {
    pub id: usize,
    pub label: String,
    pub command: String,
    pub status: ProcessStatus,
    pub pane_status: PaneStatus,
    pub tmux_address: Option<TmuxAddress>,
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
        pane_id: Option<usize>) -> Self {
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
        tmux_address: None
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
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

    pub fn mark_process_status(mut self, status: ProcessStatus, process_id: usize) -> Self {
        self.init_state.processes = self.init_state.processes.iter()
            .map(|p| {
            let mut p = p.clone();
            if p.id == process_id {
                p.status= status.clone();
            }
            p
        }).collect();
        self
    }   

    pub fn mark_pane_status(mut self, status: PaneStatus, process_id: usize) -> Self {
        self.init_state.processes = self.init_state.processes.iter()
            .map(|p| {
            let mut p = p.clone();
            if p.id == process_id {
                p.pane_status = status.clone();
            }
            p
        }).collect();
        self
    }


    pub fn set_tmux_address(mut self, addy: Option<TmuxAddress>) -> Self {
        self.init_state.processes[self.init_state.current_selection].tmux_address = addy;
        self
    }

    pub fn commit(self) -> State {
        self.init_state
    }
}




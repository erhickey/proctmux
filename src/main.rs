mod tmux;
mod config;
use std::io::{stdin, stdout, Result, Stdout, Write};

use termion::{clear, cursor, raw::IntoRawMode, style, terminal_size};
use termion::color::{self, Bg, Fg};
use termion::event::Key;
use termion::input::TermRead;

const UP: char = '▲';
const DOWN: char = '▼';

#[derive(Clone)]
enum ProcessStatus {
    Running = 1,
    Halting = 2,
    Halted = 3
}

#[derive(Clone, Eq, PartialEq)]
enum PaneStatus {
    Null = 1,
    Running = 2,
    Dead = 3
}

#[derive(Clone)]
struct Command {
    id: usize,
    label: String,
    command: String,
    status: ProcessStatus,
    pane_status: PaneStatus
}

fn create_command(id: usize, label: &str, command: &str) -> Command {
    Command
        { id
        , label: label.to_string()
        , command: command.to_string()
        , status: ProcessStatus::Halted
        , pane_status: PaneStatus::Null
        }
}

struct State {
    current_selection: usize,
    commands: Vec<Command>,
    messages: Vec<String>
}

impl State {
    fn current_command(&mut self) -> Command {
        self.commands[self.current_selection].clone()
    }

    fn break_pane(&mut self) {
        let command = self.current_command();
        if command.pane_status != PaneStatus::Null {
            tmux::break_pane(command.id, &command.label);
        }
    }

    fn join_pane(&mut self) {
        let command = self.current_command();
        if command.pane_status != PaneStatus::Null {
            tmux::join_pane(command.id);
        }
    }

    fn next_command(&mut self) {
        self.messages = vec![];
        self.break_pane();
        if self.current_selection >= self.commands.len() - 1 {
            self.current_selection = 0;
        } else {
            self.current_selection += 1;
        }
        self.join_pane();
    }

    fn previous_command(&mut self) {
        self.messages = vec![];
        self.break_pane();
        if self.current_selection <= 0 {
            self.current_selection = self.commands.len() - 1;
        } else {
            self.current_selection -= 1;
        }
        self.join_pane();
    }

    fn start_process(&mut self) {
        self.commands[self.current_selection].status = ProcessStatus::Running;
        let command = self.current_command();
        if command.pane_status == PaneStatus::Dead {
            // tmux respawn-window
        }
        if command.pane_status == PaneStatus::Null {
            self.messages = vec![format!("creating pane: {}", command.command)];
            let result = tmux::create_pane(&command.command);
            match result {
                Ok(output) => self.messages.push(format!("{}", String::from_utf8_lossy(&output.stdout))),
                Err(e) => self.messages.push(format!("{e}")),
            }
            self.commands[self.current_selection].pane_status = PaneStatus::Running;
        }
    }

    fn halt_process(&mut self) {
        self.commands[self.current_selection].status = ProcessStatus::Halted;
    }

    fn set_halting(&mut self) {
        self.commands[self.current_selection].status = ProcessStatus::Halting;
    }
}

fn main() -> Result<()> {
    let state = State {
        current_selection: 0,
        commands: vec![
            create_command(1, "Simple Echo", "echo hi"),
            create_command(2, "Echo x10", "for i in 1 2 3 4 5 6 7 8 9 10 ; do echo $i; sleep 2 ; done"),
            create_command(3, "vim", "vim")
        ],
        messages: vec![]
    };

    tmux::start_detached_session()?;
    tmux::set_remain_on_exit(true)?;

    let mut stdout = stdout().into_raw_mode()?;

    write!(stdout, "{}", cursor::Hide)?;

    draw_screen(&state, &stdout).expect("fuck");
    event_loop(state, &stdout);

    write!(stdout, "{}{}{}", cursor::Goto(0, 1), clear::All, cursor::Show)?;

    tmux::stop_detached_session()?;
    tmux::set_remain_on_exit(false)?;

    Ok(())
}

fn draw_screen(state: &State, mut stdout: &Stdout) -> Result<()> {
    write!(stdout, "{}", clear::All)?;

    for (ix, c) in state.commands.iter().enumerate() {
        match c.status {
            ProcessStatus::Running =>
                write!(
                    stdout,
                    "{}{} {} ",
                    cursor::Goto(0, (ix + 1) as u16),
                    Fg(color::Green),
                    UP
                )?,
            ProcessStatus::Halting =>
                write!(
                    stdout,
                    "{}{} {} ",
                    cursor::Goto(0, (ix + 1) as u16),
                    Fg(color::Yellow),
                    UP
                )?,
            ProcessStatus::Halted =>
                write!(
                    stdout,
                    "{}{} {} ",
                    cursor::Goto(0, (ix + 1) as u16),
                    Fg(color::Red),
                    DOWN,
                )?
        }

        if state.current_selection == ix {
            write!(
                stdout,
                "{}{}{}{:width$}{}",
                Bg(color::LightMagenta),
                Fg(color::Black),
                style::Bold,
                c.label,
                style::Reset,
                width=20
            )?;
        } else {
            write!(
                stdout,
                "{}{}{}",
                Fg(color::Cyan),
                c.label,
                style::Reset
            )?;
        }
    }

    for (ix, msg) in state.messages.iter().enumerate() {
        let (_, height) = terminal_size()?;
        write!(
            stdout,
            "{}{}{}",
            cursor::Goto(0, height - ix as u16),
            Fg(color::Red),
            msg
        )?;
    }

    stdout.flush()?;
    Ok(())
}

fn event_loop(mut state: State, mut stdout: &Stdout) {
    let stdin = stdin();

    for c in stdin.keys() {
        match c {
            Ok(Key::Char('j')) => {
                state.next_command();
                draw_screen(&state, stdout);
            },
            Ok(Key::Char('k')) => {
                state.previous_command();
                draw_screen(&state, stdout);
            },
            Ok(Key::Char('s')) => {
                state.start_process();
                draw_screen(&state, stdout);
            },
            Ok(Key::Char('x')) => {
                state.halt_process();
                draw_screen(&state, stdout);
            },
            Ok(Key::Char('h')) => {
                state.set_halting();
                draw_screen(&state, stdout);
            },
            Ok(Key::Char('q')) => {
                break;
            },
            Err(e) => {
                write!(stdout, "{}", e);
            },
            _ => {}
        }
    }
}

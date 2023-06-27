use std::io::{stdin, stdout, Result, Stdout, Write};

use termion::{clear, cursor, raw::IntoRawMode, style};
use termion::color::{self, Bg, Fg};
use termion::event::Key;
use termion::input::TermRead;

const UP: char = '▲';
const DOWN: char = '▼';

enum ProcessStatus {
    Running = 1,
    Halting = 2,
    Halted = 3
}

struct Process {
    label: String,
    command: String,
    status: ProcessStatus
}

fn create_process(label: &str, command: &str) -> Process {
    Process { label: label.to_string(), command: command.to_string(), status: ProcessStatus::Halted }
}

struct State {
    current_selection: usize,
    processes: Vec<Process>
}

impl State {
    fn next_process(&mut self) {
        if self.current_selection >= self.processes.len() - 1 {
            self.current_selection = 0;
        } else {
            self.current_selection += 1;
        }
    }

    fn previous_process(&mut self) {
        if self.current_selection <= 0 {
            self.current_selection = self.processes.len() - 1;
        } else {
            self.current_selection -= 1;
        }
    }

    fn start_process(&mut self) {
        self.processes[self.current_selection].status = ProcessStatus::Running;
    }

    fn halt_process(&mut self) {
        self.processes[self.current_selection].status = ProcessStatus::Halted;
    }

    fn set_halted(&mut self) {
        self.processes[self.current_selection].status = ProcessStatus::Halting;
    }
}

fn main() -> Result<()> {
    let state = State {
        current_selection: 0,
        processes: vec![
            create_process("Simple Echo", "echo hi"),
            create_process("Echo x10", "for i in 1 2 3 4 5 6 7 8 9 10 ; do echo $i; sleep 2 ; done"),
            create_process("vim", "vim")
        ]
    };

    let mut stdout = stdout().into_raw_mode()?;

    write!(stdout, "{}", cursor::Hide)?;

    draw_screen(&state, &stdout).expect("fuck");
    event_loop(state, &stdout);

    write!(stdout, "{}{}{}", cursor::Goto(0, 1), clear::All, cursor::Show)?;
    Ok(())
}

fn draw_screen(state: &State, mut stdout: &Stdout) -> Result<()> {
    write!(stdout, "{}", clear::All)?;

    for (ix, p) in state.processes.iter().enumerate() {
        match p.status {
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
                p.label,
                style::Reset,
                width=20
            )?;
        } else {
            write!(
                stdout,
                "{}{}{}",
                Fg(color::Cyan),
                p.label,
                style::Reset
            )?;
        }
    }

    stdout.flush()?;
    Ok(())
}

fn event_loop(mut state: State, mut stdout: &Stdout) {
    let stdin = stdin();

    for c in stdin.keys() {
        match c {
            Ok(Key::Char('j')) => {
                state.next_process();
                draw_screen(&state, stdout);
            },
            Ok(Key::Char('k')) => {
                state.previous_process();
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
                state.set_halted();
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

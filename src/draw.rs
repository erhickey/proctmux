use std::io::{stdout, Stdout, Write};

use termion::{clear, cursor, style, terminal_size};
use termion::color::{self, Bg, Fg};
use termion:: raw::{IntoRawMode, RawTerminal};

use crate::model::{ProcessStatus, State};


const UP: char = '▲';
const DOWN: char = '▼';

pub fn init_screen() -> Result<RawTerminal<Stdout>, Box<dyn std::error::Error>> {
    let mut stdout = stdout().into_raw_mode()?;
    write!(stdout, "{}", cursor::Hide)?;
    Ok(stdout)
}

pub fn prepare_screen_for_exit(mut stdout: &Stdout) -> Result<(), Box<dyn std::error::Error>> {
    write!(
        stdout,
        "{}{}{}",
        cursor::Goto(0, 1),
        clear::All,
        cursor::Show
    )?;
    Ok(())
}

pub fn draw_screen(state: &State, mut stdout: &Stdout) -> Result<(), Box<dyn std::error::Error>> {
    write!(stdout, "{}", clear::All)?;

    for (ix, c) in state.commands.iter().enumerate() {
        match c.status {
            ProcessStatus::Running => write!(
                stdout,
                "{}{} {} ",
                cursor::Goto(0, (ix + 1) as u16),
                Fg(color::Green),
                UP
            )?,
            ProcessStatus::Halting => write!(
                stdout,
                "{}{} {} ",
                cursor::Goto(0, (ix + 1) as u16),
                Fg(color::Yellow),
                UP
            )?,
            ProcessStatus::Halted => write!(
                stdout,
                "{}{} {} ",
                cursor::Goto(0, (ix + 1) as u16),
                Fg(color::Red),
                DOWN,
            )?,
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
                width = 20
            )?;
        } else {
            write!(stdout, "{}{}{}", Fg(color::Cyan), c.label, style::Reset)?;
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

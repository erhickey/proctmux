use std::error::Error;
use std::io::{Stdout, Write, stdout};

use termion::{clear, cursor, style, terminal_size};
use termion::color::{Bg, Fg, self};
use termion:: raw::{IntoRawMode, RawTerminal};

use crate::model::{ProcessStatus, State};

const UP: char = '▲';
const DOWN: char = '▼';
const RIGHT: char = '▶';

pub fn init_screen() -> Result<RawTerminal<Stdout>, Box<dyn Error>> {
    let mut stdout = stdout().into_raw_mode()?;
    write!(stdout, "{}", cursor::Hide)?;
    Ok(stdout)
}

pub fn prepare_screen_for_exit(mut stdout: &Stdout) -> Result<(), Box<dyn Error>> {
    write!(
        stdout,
        "{}{}{}",
        cursor::Goto(0, 1),
        clear::All,
        cursor::Show
    )?;
    Ok(())
}

pub fn draw_screen(mut stdout: &Stdout, state: &State) -> Result<(), Box<dyn Error>> {
    write!(stdout, "{}", clear::All)?;
    let mut y_offset = 1;
    if state.gui_state.entering_filter_text {
        let filter_text = state.gui_state.filter_text.clone().unwrap_or("".to_string());
            write!(stdout, "{}{}{} {}{}{}", 
                cursor::Goto(0, y_offset as u16),
                Fg(color::White),
                style::Bold,
                RIGHT,
                filter_text, 
                style::Reset)?;
        y_offset += 1;
    }

    for (ix, c) in state.get_filtered_processes()
        .iter()
        .enumerate() {
        let y_pos = ix + y_offset;
        match c.status {
            ProcessStatus::Running => write!(
                stdout,
                "{}{} {} ",
                cursor::Goto(0, (y_pos + 1) as u16),
                Fg(color::Green),
                UP
            )?,
            ProcessStatus::Halting => write!(
                stdout,
                "{}{} {} ",
                cursor::Goto(0, (y_pos + 1) as u16),
                Fg(color::Yellow),
                DOWN
            )?,
            ProcessStatus::Halted => write!(
                stdout,
                "{}{} {} ",
                cursor::Goto(0, (y_pos + 1) as u16),
                Fg(color::Red),
                DOWN,
            )?,
        }

        if state.current_proc_id == c.id {
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

    for (ix, msg) in state.gui_state.messages.iter().enumerate() {
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

use std::error::Error;
use std::io::{stdout, Stdout, Write};

use termion::color::{self, Bg, Color, Fg};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, cursor, style, terminal_size};

use crate::model::{Process, ProcessStatus, State};

const UP: char = '▲';
const DOWN: char = '▼';
const RIGHT: char = '▶';
static ANSI_PREFIX: &str = "ansi";

fn color_from_config_string(s: &str) -> Result<Box<dyn Color>, Box<dyn Error>> {
    // TODO there might be a better way to do this.
    // i was trying to retain backward compatibility with procmux config as much as possible
    // which this does, but i admit its fugly
    let config_str = s.to_lowercase();
    if config_str.starts_with(ANSI_PREFIX) {
        let config_str = config_str.trim_start_matches(ANSI_PREFIX);
        let c: Box<dyn Color> = match config_str {
            "red" => Box::new(color::Red),
            "green" => Box::new(color::Green),
            "blue" => Box::new(color::Blue),
            "yellow" => Box::new(color::Yellow),
            "cyan" => Box::new(color::Cyan),
            "magenta" => Box::new(color::Magenta),
            "black" => Box::new(color::Black),
            "white" => Box::new(color::White),
            "lightred" => Box::new(color::LightRed),
            "lightgreen" => Box::new(color::LightGreen),
            "lightblue" => Box::new(color::LightBlue),
            "lightyellow" => Box::new(color::LightYellow),
            "lightcyan" => Box::new(color::LightCyan),
            "lightmagenta" => Box::new(color::LightMagenta),
            "lightblack" => Box::new(color::LightBlack),
            "lightwhite" => Box::new(color::LightWhite),
            _ => return Err(Box::from("Unknown color")),
        };
        return Ok(c);
    }
    let mut parts = config_str.split(",");
    let r = parts.next().unwrap().parse::<u8>()?;
    let g = parts.next().unwrap().parse::<u8>()?;
    let b = parts.next().unwrap().parse::<u8>()?;
    Ok(Box::new(color::Rgb(r, g, b)))
}

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

fn get_status_arrow_and_color(state: &State, process: &Process) -> (Box<dyn Color>, char) {
    match process.status {
        ProcessStatus::Running => {
            let fg = color_from_config_string(&state.config.style.status_running_color)
                .unwrap_or(Box::new(color::Green));
            (fg, UP)
        }
        ProcessStatus::Halting => {
            let fg = color_from_config_string(&state.config.style.status_halting_color)
                .unwrap_or(Box::new(color::Yellow));
            (fg, DOWN)
        }
        ProcessStatus::Halted => {
            let fg = color_from_config_string(&state.config.style.status_stopped_color)
                .unwrap_or(Box::new(color::Red));
            (fg, DOWN)
        }
    }
}

pub fn print_messages(
    mut stdout: &Stdout,
    msgs: &[(Box<dyn Color>, String)],
) -> Result<(), Box<dyn Error>> {
    let (_, height) = terminal_size()?;
    for (idx, color_and_msg) in msgs.iter().enumerate() {
        let (color, msg) = color_and_msg;
        write!(
            stdout,
            "{}{}{}",
            cursor::Goto(0, height - idx as u16),
            Fg(color.as_ref()),
            msg
        )?;
    }
    Ok(())
}
pub fn draw_screen(mut stdout: &Stdout, state: &State) -> Result<(), Box<dyn Error>> {
    write!(stdout, "{}", clear::All)?;
    let mut y_offset = 1;
    if state.gui_state.entering_filter_text {
        let filter_text = state
            .gui_state
            .filter_text
            .clone()
            .unwrap_or("".to_string());
        write!(
            stdout,
            "{}{}{} {}{}{}",
            cursor::Goto(0, y_offset as u16),
            Fg(color::White),
            style::Bold,
            RIGHT,
            filter_text,
            style::Reset
        )?;
        y_offset += 1;
    }

    for (ix, proc) in state.get_filtered_processes().iter().enumerate() {
        let y_pos = ix + y_offset;
        let (fg, arrow) = get_status_arrow_and_color(state, proc);
        write!(
            stdout,
            "{}{} {} ",
            cursor::Goto(0, (y_pos + 1) as u16),
            Fg(fg.as_ref()),
            arrow
        )?;

        if state.current_proc_id == proc.id {
            let bg = color_from_config_string(&state.config.style.selected_process_bg_color)
                .unwrap_or(Box::new(color::LightMagenta));
            let fg = color_from_config_string(&state.config.style.selected_process_color)
                .unwrap_or(Box::new(color::Black));
            write!(
                stdout,
                "{}{}{}{:width$}{}",
                Bg(bg.as_ref()),
                Fg(fg.as_ref()),
                style::Bold,
                proc.label,
                style::Reset,
                width = state.config.layout.process_list_width
            )?;
        } else {
            let fg = color_from_config_string(&state.config.style.unselected_process_color)
                .unwrap_or(Box::new(color::Cyan));
            write!(stdout, "{}{}{}", Fg(fg.as_ref()), proc.label, style::Reset)?;
        }
    }
    let current_proc = state.current_process();
    let mut all_msgs: Vec<(Box<dyn Color>, String)> = vec![];
    // add process descriptions / short-help text
    if !state.config.layout.hide_help {
        if let Some(current_proc) = current_proc {
            let desc = &current_proc.config.description;
            all_msgs.push((Box::new(color::White) as Box<dyn Color>, desc.clone().unwrap_or("".to_string())));
        }
    }
    // add error messages
    state
        .gui_state
        .messages
        .iter()
        .map(|m| (Box::new(color::Red) as Box<dyn Color>, m.to_string()))
        .for_each(|m| all_msgs.push(m));
    print_messages(stdout, all_msgs.as_slice())?;

    stdout.flush()?;
    Ok(())
}

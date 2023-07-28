use std::error::Error;
use std::io::{stdout, Stdout, Write};

use termion::color::{self, Bg, Color, Fg};
use termion::event::Key;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, cursor, style, terminal_size};

use crate::process::{Process, ProcessStatus};
use crate::state::State;

const UP: char = '▲';
const DOWN: char = '▼';
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
fn key_to_str(key: &Key) -> String {
    fn to_printable_char(c: &char) -> char {
        if *c == '\n' {
            '↵'
        } else {
            *c
        }
    }
    match key {
        Key::Char(c) => format!("{}", to_printable_char(c)),
        Key::Alt(c) => format!("⎇-{}", to_printable_char(c)),
        Key::Ctrl(c) =>  format!("^-{}", to_printable_char(c)),
        Key::Left => "←".to_string(),
        Key::Right => "→".to_string(),
        Key::Up => "↑".to_string(),
        Key::Down => "↓".to_string(),
        Key::Backspace => "⎵".to_string(),
        Key::Delete => "⌫".to_string(),
        Key::End => "end".to_string(),
        Key::Esc => "esc".to_string(),
        Key::F(i) => format!("fn-{}", i),
        Key::Home => "home".to_string(),
        Key::Insert => "ins".to_string(),
        Key::Null => "null".to_string(),
        Key::PageDown => "pgdn".to_string(),
        Key::PageUp => "pgup".to_string(),
        Key::BackTab => "backtab".to_string(),
        Key::__IsNotComplete => "".to_string(),
    }
}
fn keys_to_str(keys: &[Key]) -> String {
    keys.iter()
        .map(key_to_str)
        .collect::<Vec<String>>()
        .join(", ")
}
fn condense_to_width(width: usize, s: &str) -> Vec<String> {
    let condensed = s
        .chars()
        .collect::<Vec<char>>()
        .chunks(width)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<String>>();
    condensed
}

pub fn get_help_messages(state: &State) -> Vec<(Box<dyn Color>, String)> {
    let mut msg: Vec<String> = vec![];
    let keybindings = &state.config.keybinding;
    msg.push(format!(
        "<{}> {}",
        keys_to_str(keybindings.quit.as_slice()),
        "quit"
    ));
    msg.push(format!(
        "<{}> {}",
        keys_to_str(keybindings.start.as_slice()),
        "start"
    ));
    msg.push(format!(
        "<{}> {}",
        keys_to_str(keybindings.stop.as_slice()),
        "stop"
    ));
    msg.push(format!(
        "<{}> {}",
        keys_to_str(keybindings.up.as_slice()),
        "up"
    ));
    msg.push(format!(
        "<{}> {}",
        keys_to_str(keybindings.down.as_slice()),
        "down"
    ));
    msg.push(format!(
        "<{}> {}",
        keys_to_str(keybindings.filter.as_slice()),
        "filter"
    ));
    msg.push(format!(
        "<{}> {}",
        keys_to_str(keybindings.filter_submit.as_slice()),
        "filter_submit"
    ));
    msg.push(format!(
        "<{}> {}",
        keys_to_str(keybindings.switch_focus.as_slice()),
        "switch_focus"
    ));

    msg = msg.iter().fold(vec![], |mut a, b| {
        let last = a.last();
        if let Some(last) = last {
            let merged = format!("{} | {}", last, b);
            if merged.len() > state.config.layout.process_list_width {
                a.push(b.clone());
            } else {
                a.remove(a.len() - 1);
                a.push(merged);
            }
        } else {
            a.push(b.clone());
        }
        a
    });

    msg.iter()
        .map(|msg| (Box::new(color::White) as Box<dyn Color>, msg.clone()))
        .collect::<Vec<(Box<dyn Color>, String)>>()
}
pub fn print_messages(
    state: &State,
    mut stdout: &Stdout,
    msgs: &[(Box<dyn Color>, String)],
) -> Result<(), Box<dyn Error>> {
    let default_color = Box::new(color::White) as Box<dyn Color>;
    let (_, height) = terminal_size()?;
    let mut msgs = msgs
        .iter()
        .flat_map(|col_msg| {
            let (color, msg) = col_msg;
            let colors_and_msgs = condense_to_width(state.config.layout.process_list_width, msg)
                .iter()
                .map(|msg| (color, msg.clone()))
                .collect::<Vec<_>>();
            colors_and_msgs
        })
        .collect::<Vec<_>>();
    
    // we should never hit this but just a safeguard so that
    // if the msg array grows larger than a u16 we don't
    // fail to cast the len/indx to a u16 below 
    if msgs.len() >  u16::MAX as usize {
        msgs = msgs[0..(u16::MAX as usize -1)].to_vec();
        msgs.push((&default_color, "...".to_string()));
    }

    for (idx, color_and_msg) in msgs.iter().enumerate() {
        let (color, msg) = color_and_msg;
        write!(
            stdout,
            "{}{}{}",
            cursor::Goto(0, height - msgs.len() as u16 + idx as u16),
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
            state.config.style.pointer_char,
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
    if let Some(current_proc) = current_proc {
        if !state.config.layout.hide_process_description_panel {
            let desc = &current_proc.config.description;
            all_msgs.push((
                Box::new(color::White) as Box<dyn Color>,
                desc.clone().unwrap_or("".to_string()),
            ));
        }
    }

    if !state.config.layout.hide_help {
        all_msgs.extend(get_help_messages(state));
    }
    // add error messages
    state
        .gui_state
        .messages
        .iter()
        .map(|m| (Box::new(color::Red) as Box<dyn Color>, m.to_string()))
        .for_each(|m| all_msgs.push(m));
    print_messages(state, stdout, all_msgs.as_slice())?;

    stdout.flush()?;
    Ok(())
}

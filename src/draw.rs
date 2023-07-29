use std::error::Error;
use std::io::{stdout, Stdout, Write};

use termion::color::{self, Bg, Color, Fg};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, cursor, style, terminal_size};

use crate::frame::{
    break_at_natural_break_points, wrap_lines_to_width, wrap_to_width, ColoredSegment,
};
use crate::repr::{color_from_config_string, get_status_arrow_and_color, keybinding_help};
use crate::state::State;

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

pub fn get_help_messages(state: &State) -> Vec<ColoredSegment> {
    let mut msg: Vec<String> = vec![];
    let keybindings = &state.config.keybinding;
    msg.push(keybinding_help(keybindings.quit.as_slice(), "quit"));
    msg.push(keybinding_help(keybindings.start.as_slice(), "start"));
    msg.push(keybinding_help(keybindings.stop.as_slice(), "stop"));
    msg.push(keybinding_help(keybindings.up.as_slice(), "up"));
    msg.push(keybinding_help(keybindings.down.as_slice(), "down"));
    msg.push(keybinding_help(keybindings.filter.as_slice(), "filter"));
    msg.push(keybinding_help(
        keybindings.filter_submit.as_slice(),
        "submit filter",
    ));
    msg.push(keybinding_help(
        keybindings.switch_focus.as_slice(),
        "switch focus",
    ));

    // try to make as may keybindings fit on one line as possible.
    // once the line length exceeds the process list width, start a new line.
    msg = msg.iter().fold(vec![], |acc, next| {
        break_at_natural_break_points(state.config.layout.process_list_width, " | ", acc, next)
    });

    msg.iter()
        .map(|msg| ColoredSegment::new_basic(Box::new(color::White) as Box<dyn Color>, msg.clone()))
        .collect::<Vec<ColoredSegment>>()
}
pub fn print_messages(
    state: &State,
    mut stdout: &Stdout,
    msgs: &[ColoredSegment],
) -> Result<(), Box<dyn Error>> {
    // let default_color = Box::new(color::White) as Box<dyn Color>;
    let (_, height) = terminal_size()?;
    for (idx, colored_segment) in msgs.iter().enumerate() {
        write!(
            stdout,
            "{}{}{}",
            cursor::Goto(0, height - msgs.len() as u16 + idx as u16),
            Fg(colored_segment.fg.as_ref()),
            colored_segment.text
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
        let arrow_segment = get_status_arrow_and_color(state, proc);
        write!(
            stdout,
            "{}{} {} ",
            cursor::Goto(0, (y_pos + 1) as u16),
            Fg(arrow_segment.fg.as_ref()),
            arrow_segment.text
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
    let mut all_msgs: Vec<ColoredSegment> = vec![];

    // add process descriptions / short-help text
    if let Some(current_proc) = current_proc {
        if !state.config.layout.hide_process_description_panel {
            let desc = &current_proc.config.description;
            if let Some(desc) = desc {
                let desc_msgs = wrap_to_width(state.config.layout.process_list_width, desc);
                for msg in desc_msgs {
                    all_msgs.push(ColoredSegment::new_basic(
                        Box::new(color::White) as Box<dyn Color>,
                        msg.clone(),
                    ));
                }
            }
        }
    }

    if !state.config.layout.hide_help {
        all_msgs.extend(get_help_messages(state));
    }
    // add error messages
    wrap_lines_to_width(
        state.config.layout.process_list_width,
        &state.gui_state.messages,
    )
    .iter()
    .map(|m| ColoredSegment::new_basic(Box::new(color::Red) as Box<dyn Color>, m.to_string()))
    .for_each(|m| all_msgs.push(m));

    print_messages(state, stdout, all_msgs.as_slice())?;

    stdout.flush()?;
    Ok(())
}

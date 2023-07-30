use std::error::Error;
use std::fmt::Display;
use std::io::{stdout, Stdout, Write};

use termion::color::{self, Bg, Color, Fg};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, cursor, style, terminal_size};

use crate::frame::{
    break_at_natural_break_points, wrap_lines_to_width, wrap_to_width, ColoredSegment,
    ProcessPanelFrame,
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

fn get_filter_frame_line(state: &State) -> Option<Vec<ColoredSegment>> {
    if state.gui_state.entering_filter_text {
        let filter_text = state
            .gui_state
            .filter_text
            .clone()
            .unwrap_or("".to_string());
        let line = vec![
            ColoredSegment::new_basic(
                Box::new(color::White) as Box<dyn Color>,
                state.config.style.pointer_char.clone(),
            ),
            ColoredSegment::new_basic(Box::new(color::White) as Box<dyn Color>, filter_text),
        ];

        return Some(line);
    }
    None
}

fn get_process_lines(state: &State) -> Vec<Vec<ColoredSegment>> {
    let process_label_width = state.config.layout.process_list_width - 3;

    let mut lines: Vec<Vec<ColoredSegment>> = vec![];
    for proc in state.get_filtered_processes().iter() {
        if state.current_proc_id == proc.id {
            let bg = color_from_config_string(&state.config.style.selected_process_bg_color)
                .unwrap_or(Box::new(color::LightMagenta));
            let fg = color_from_config_string(&state.config.style.selected_process_color)
                .unwrap_or(Box::new(color::Black));
            let line = vec![
                get_status_arrow_and_color(state, proc),
                ColoredSegment::new_basic(
                    Box::new(color::White) as Box<dyn Color>,
                    " ".to_string(),
                ),
                ColoredSegment::new_basic(fg as Box<dyn Color>, proc.label.clone())
                    .set_bg(bg)
                    .set_style(Box::new(style::Bold) as Box<dyn Display>)
                    .set_width(process_label_width),
            ];
            lines.push(line);
        } else {
            let fg = color_from_config_string(&state.config.style.unselected_process_color)
                .unwrap_or(Box::new(color::Cyan));
            let line = vec![
                get_status_arrow_and_color(state, proc),
                ColoredSegment::new_basic(
                    Box::new(color::White) as Box<dyn Color>,
                    " ".to_string(),
                ),
                ColoredSegment::new_basic(fg as Box<dyn Color>, proc.label.clone()),
            ];
            lines.push(line);
        }
    }
    lines
}

fn get_message_lines(state: &State) -> Vec<ColoredSegment> {
    let process_list_width = state.config.layout.process_list_width;
    let current_proc = state.current_process();
    let mut all_msgs: Vec<ColoredSegment> = vec![];

    // add process descriptions / short-help text
    if let Some(current_proc) = current_proc {
        if !state.config.layout.hide_process_description_panel {
            let desc = &current_proc.config.description;
            if let Some(desc) = desc {
                let desc_msgs = wrap_to_width(process_list_width, desc);
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
    wrap_lines_to_width(process_list_width, &state.gui_state.messages)
        .iter()
        .map(|m| ColoredSegment::new_basic(Box::new(color::Red) as Box<dyn Color>, m.to_string()))
        .for_each(|m| all_msgs.push(m));

    all_msgs
}
pub fn construct_frame(state: &State) -> ProcessPanelFrame {
    let mut frame = ProcessPanelFrame::new(state.config.layout.process_list_width);
    frame.set_filter_line(get_filter_frame_line(state));
    frame.set_process_lines(get_process_lines(state));
    frame.set_messages(get_message_lines(state));
    frame
}

pub fn draw_colored_segment(
    mut stdout: &Stdout,
    seg: &ColoredSegment,
) -> Result<(), Box<dyn Error>> {
    write!(stdout, "{}", Fg(seg.fg.as_ref()))?;
    if let Some(bg) = &seg.bg {
        write!(stdout, "{}", Bg(bg.as_ref()))?;
    }
    if let Some(style) = &seg.style {
        write!(stdout, "{}", style)?;
    }
    if let Some(width) = seg.width {
        write!(stdout, "{:width$}", seg.text, width = width)?;
    } else {
        write!(stdout, "{}", seg.text)?;
    }
    write!(stdout, "{}", style::Reset)?;
    Ok(())
}

fn draw_frame(mut stdout: &Stdout, frame: &ProcessPanelFrame) -> Result<(), Box<dyn Error>> {
    // TODO - check for terminal height make sure things will fit (truncate / ellipsize if necessary)
    // TODO - maybe a fallback condition to draw an error if the screen is too small for displaying anything
    // TODO - maybe simulate scrolling if the process list is too long
    let (_, height) = terminal_size()?;
    fn goto_from_top(mut stdout: &Stdout, y: u16) -> Result<(), Box<dyn Error>> {
        write!(stdout, "{}", cursor::Goto(0, y))?;
        Ok(())
    }
    fn goto_from_bottom(mut stdout: &Stdout, height: u16, y: u16) -> Result<(), Box<dyn Error>> {
        write!(stdout, "{}", cursor::Goto(0, height - y))?;
        Ok(())
    }
    write!(stdout, "{}", clear::All)?;
    let mut y_offset: u16 = 1;
    if let Some(filter_line) = &frame.filter_line {
        goto_from_top(stdout, y_offset)?;
        for seg in filter_line {
            draw_colored_segment(stdout, seg)?;
        }
        y_offset += 1;
    }
    for line in frame.process_lines.iter() {
        goto_from_top(stdout, y_offset)?;
        for seg in line {
            draw_colored_segment(stdout, seg)?;
        }
        y_offset += 1;
    }
    for (idx, seg) in frame.messages.iter().enumerate() {
        let y_offest_from_bottom = frame.messages.len() as u16 - idx as u16;
        goto_from_bottom(stdout, height, y_offest_from_bottom)?;
        draw_colored_segment(stdout, seg)?;
    }
    stdout.flush()?;
    Ok(())
}
pub fn draw_screen(stdout: &Stdout, state: &State) -> Result<(), Box<dyn Error>> {
    let frame = construct_frame(state);
    draw_frame(stdout, &frame)?;
    Ok(())
}

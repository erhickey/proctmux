use std::fmt::Display;

use termion::color::Color;

pub struct ProcessPanelFrame {
    pub width: usize,
    pub filter_line: Option<Vec<ColoredSegment>>,
    pub process_lines: Vec<Vec<ColoredSegment>>,
    pub messages: Vec<ColoredSegment>,
}
impl ProcessPanelFrame {
    pub fn new(width: usize) -> Self {
        Self {
            width,
            filter_line: None,
            process_lines: vec![],
            messages: vec![],
        }
    }
    pub fn set_filter_line(&mut self, filter_line: Option<Vec<ColoredSegment>>) -> &mut Self {
        self.filter_line = filter_line;
        self
    }
    pub fn set_process_lines(&mut self, process_lines: Vec<Vec<ColoredSegment>>) -> &mut Self {
        self.process_lines = process_lines;
        self
    }
    pub fn set_messages(&mut self, messages: Vec<ColoredSegment>) -> &mut Self {
        self.messages = messages;
        self
    }
}

pub fn wrap_lines_to_width(width: usize, msgs: &[String]) -> Vec<String> {
    msgs.iter()
        .flat_map(|line| {
            let extended_lines = wrap_to_width(width, line)
                .iter()
                .map(|x| x.to_owned())
                .collect::<Vec<_>>();
            extended_lines
        })
        .collect::<Vec<_>>()
}
pub fn wrap_to_width(width: usize, s: &str) -> Vec<String> {
    let condensed = s
        .chars()
        .collect::<Vec<char>>()
        .chunks(width)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<String>>();
    condensed
}
pub fn break_at_natural_break_points(
    width: usize,
    delimiter: &str,
    mut acc: Vec<String>,
    next: &str,
) -> Vec<String> {
    let last = acc.last();
    if let Some(last) = last {
        let merged = format!("{}{}{}", last, delimiter, next);
        if merged.len() > width {
            acc.push(next.to_string());
        } else {
            acc.remove(acc.len() - 1);
            acc.push(merged);
        }
    } else {
        acc.push(next.to_string());
    }
    acc
}
pub struct ColoredSegment {
    pub fg: Box<dyn Color>,
    pub bg: Option<Box<dyn Color>>,
    pub text: String,
    pub style: Option<Box<dyn Display>>,
    pub width: Option<usize>,
}

impl ColoredSegment {
    pub fn new_basic(fg: Box<dyn Color>, text: String) -> Self {
        Self {
            fg,
            bg: None,
            text,
            style: None,
            width: None,
        }
    }
    pub fn set_bg(mut self, bg: Box<dyn Color>) -> Self {
        self.bg = Some(bg);
        self
    }
    pub fn set_style(mut self, style: Box<dyn Display>) -> Self {
        self.style = Some(style);
        self
    }
    pub fn set_width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }
}

use termion::color::Color;

pub struct ProcessPanelFrame {
    filter_line: Option<String>,
    process_lines: Vec<String>,
    messages: Vec<ColoredSegment>,
}

pub fn wrap_lines_to_width(width: usize, msgs: &Vec<String>) -> Vec<String> {
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
    bg: Option<Box<dyn Color>>,
    pub text: String,
    style: Option<String>,
}

impl ColoredSegment {
    pub fn new_basic(fg: Box<dyn Color>, text: String) -> Self {
        Self {
            fg,
            bg: None,
            text,
            style: None,
        }
    }
}

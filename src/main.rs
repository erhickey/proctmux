use std::io::{stdout, Write, Stdout};

use futures::StreamExt;

use crossterm::{
    cursor, execute, queue, Result,
    event::{Event, EventStream, KeyCode},
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, enable_raw_mode, disable_raw_mode}
};

struct State {
    current_selection: usize,
    processes: Vec<(String, String)>
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
}

fn main() -> Result<()> {
    let state = State {
        current_selection: 0,
        processes: vec![
            ("Simple Echo".to_string(), "echo hi".to_string()),
            ("Echo x10".to_string(), "for i in 1 2 3 4 5 6 7 8 9 10 ; do echo $i; sleep 2 ; done".to_string()),
            ("vim".to_string(), "vim".to_string())
        ]
    };

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, cursor::Hide)?;

    draw_screen(&state, &stdout)?;
    async_std::task::block_on(event_loop(state, &stdout));

    execute!(stdout, cursor::Show)?;
    disable_raw_mode()?;
    Ok(())
}

fn draw_screen(state: &State, mut stdout: &Stdout) -> Result<()> {
    const UP: char = '▲';
    const DOWN: char = '▼';

    queue!(stdout, terminal::Clear(terminal::ClearType::All))?;

    for (ix, (label, _)) in state.processes.iter().enumerate() {
        if state.current_selection == ix {
            queue!(
                stdout,
                cursor::MoveTo(0, ix as u16),
                SetForegroundColor(Color::Green),
                SetBackgroundColor(Color::Blue),
                Print(format!(" {} ", UP)),
                SetForegroundColor(Color::Magenta),
                Print(format!("{:width$}", label, width=30)),
                ResetColor
            )?;
        } else {
            queue!(
                stdout,
                cursor::MoveTo(0, ix as u16),
                SetForegroundColor(Color::Red),
                Print(format!(" {} ", DOWN)),
                SetForegroundColor(Color::Cyan),
                Print(label),
                ResetColor
            )?;
        }
    }

    stdout.flush()?;
    Ok(())
}

async fn event_loop(mut state: State, stdout: &Stdout) {
    let mut reader = EventStream::new();

    loop {
        match reader.next().await {
            Some(Ok(event)) => {
                if event == Event::Key(KeyCode::Char('j').into()) {
                    state.next_process();
                    draw_screen(&state, stdout);
                }

                if event == Event::Key(KeyCode::Char('k').into()) {
                    state.previous_process();
                    draw_screen(&state, stdout);
                }

                if event == Event::Key(KeyCode::Char('q').into()) || event == Event::Key(KeyCode::Esc.into()) {
                    break;
                }
            }
            Some(Err(e)) => println!("Error: {:?}\r", e),
            None => break,
        }
    }
}

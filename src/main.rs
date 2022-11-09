use std::{io, io::Write, thread, time::Duration, process::exit, fs, env};
use tui::{
    backend::CrosstermBackend,
    widgets::{Widget, Block, Borders, Paragraph, Wrap},
    style::{Style, Color},
    layout::{Layout, Constraint, Direction, Alignment},
    Terminal,
    text::{Span, Spans}
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, KeyEvent, read},
    execute,
    cursor,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

fn main() -> Result<(), io::Error> {
    // setup terminal
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1{
        exit(1);
    }
    let file_path = &args[1];
    let file_path_span = Span::raw(String::from(file_path));
    let file_string = fs::read_to_string(file_path).expect("File not able to be read");
    let mut file_vector: Vec<&str> = file_string.lines().collect();
    let mut line_count: u16 = file_vector.len() as u16;
    let mut line_lengths: Vec<u16> = Vec::new();
    for i in 0..line_count{
        line_lengths.push(file_vector[i as usize].len() as u16);
    }
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut display_text = vec![Spans::from(file_vector[0])];
    for i in 1..line_count{
        display_text.push(Spans::from(file_vector[i as usize]));
    }

    terminal.draw(|f| {
        let size = f.size();
        let block = Block::default()
            .title(file_path_span)
            .borders(Borders::ALL);
        let paragraph = Paragraph::new(display_text.clone())
            .style(Style::default())
            .alignment(Alignment::Left)
            .wrap(Wrap {trim: true });
        f.render_widget(block, size);
    })?;
    terminal.set_cursor(*line_lengths.last().unwrap(),line_count);
    terminal.show_cursor();

    loop{
        match read().unwrap() {
            Event::Key(KeyEvent{
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
                ..}
            ) => break,
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
                ..}
            ) => cursorMove(&mut terminal, 'n'),
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                ..}
            ) => cursorMove(&mut terminal, 's'),
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE,
                ..}
            ) => cursorMove(&mut terminal, 'e'),
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE,
                ..}
            ) => cursorMove(&mut terminal, 'w'),
            _ => (),
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn cursorMove<B: tui::backend::Backend>(terminal: &mut Terminal<B>, mut direction: char){
    let mut x_pos = terminal.get_cursor().unwrap().0;
    let mut y_pos = terminal.get_cursor().unwrap().1;
    if x_pos == 0 || x_pos == 1 && direction == 'w'{direction = 'q'}
    if y_pos == 0 || y_pos == 1 && direction == 'n'{direction = 'q'}
    match direction{
        'n' => terminal.set_cursor(x_pos, y_pos - 1),
        's' => terminal.set_cursor(x_pos, y_pos + 1),
        'e' => terminal.set_cursor(x_pos + 1, y_pos),
        'w' => terminal.set_cursor(x_pos - 1, y_pos),
        _ => Ok(()),
    };
}

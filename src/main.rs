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
    file_vector.push("");
    let mut line_lengths: Vec<u16> = Vec::new();
    for i in 0..file_vector.len(){
        line_lengths.push(file_vector[i as usize].len() as u16);
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut span_start: usize = 0;
    let mut span_changed = (false, 0 as u16, 0 as u16);
    let display_text = str_vec_to_span(file_vector.clone(), span_start.clone());

    let mut render_height: usize = 0;
    render_height = render(&mut terminal, display_text.clone(), file_path_span.clone());
    terminal.set_cursor(1, (render_height.clone() as u16) - 1);
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
            ) => span_changed = cursor_move(&mut terminal, line_lengths.clone(), 'n', render_height.clone(), &mut span_start),
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                ..}
            ) => span_changed = cursor_move(&mut terminal, line_lengths.clone(), 's', render_height.clone(), &mut span_start),
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE,
                ..}
            ) => span_changed = cursor_move(&mut terminal, line_lengths.clone(), 'e', render_height.clone(), &mut span_start),
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE,
                ..}
            ) => span_changed = cursor_move(&mut terminal, line_lengths.clone(), 'w', render_height.clone(), &mut span_start),
            _ => (),
        }
        if span_changed.0{
            let display_text = str_vec_to_span(file_vector.clone(), span_start.clone());
            render(&mut terminal, display_text.clone(), file_path_span.clone());
            let mut x_pos = span_changed.1;
            let mut y_pos = span_changed.2;
            terminal.set_cursor(x_pos, y_pos);
            terminal.show_cursor();
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

fn cursor_move<B: tui::backend::Backend>(terminal: &mut Terminal<B>, line_lengths: Vec<u16>, mut direction: char, render_height: usize, span_start: &mut usize) -> (bool, u16, u16){
    let mut x_pos = terminal.get_cursor().unwrap().0;
    let mut y_pos = terminal.get_cursor().unwrap().1;
    if y_pos <= 2 && direction == 'n' && (*span_start as usize) > 0{
        if x_pos > line_lengths[(y_pos) as usize + *span_start] + 1{
            x_pos = line_lengths[(y_pos) as usize + *span_start] + 1;
        }
        *span_start -= 1;
        return (true, x_pos, y_pos);
    }
    if y_pos <= (line_lengths.len() - *span_start) as u16{
        if y_pos >= render_height as u16 && (line_lengths.len() - (*span_start as usize)) > render_height && direction == 's'{
            if x_pos > line_lengths[(y_pos) as usize + *span_start] + 1{
                x_pos = line_lengths[(y_pos) as usize + *span_start] + 1;
            }
            *span_start += 1;
            return (true, x_pos, y_pos);
        }
    }
    if x_pos <= 1 && direction == 'w'{direction = 'q'}
    if y_pos <= 1 && direction == 'n'{direction = 'q'}
    match direction{
        'n' => y_pos -= 1,
        's' => y_pos += 1,
        'e' => x_pos += 1,
        'w' => x_pos -= 1,
        _ => (),
    }
    if y_pos > (line_lengths.len() - *span_start) as u16{
        y_pos = (line_lengths.len() - *span_start) as u16;
    }
    if x_pos > line_lengths[(y_pos - 1) as usize + *span_start] + 1{
        x_pos = line_lengths[(y_pos - 1) as usize + *span_start] + 1;
    }
    terminal.set_cursor(x_pos, y_pos);
    return (false, x_pos, y_pos);
}
fn str_vec_to_span(file_vector: Vec<&str>, start_y: usize) -> Vec<Spans<'_>>{
    let mut display_text = vec![Spans::from(file_vector[start_y])];
    for i in (start_y + 1)..file_vector.len(){
        display_text.push(Spans::from(file_vector[i as usize]));
    }
    return display_text
}
fn render<B: tui::backend::Backend>(terminal: &mut Terminal<B>, display_text: Vec<Spans<'_>>, file_path_span: Span) -> usize{
    let mut return_size: usize = 0;
    terminal.draw(|main| {
        let mut size = main.size();
        let block = Block::default()
            .title(file_path_span)
            .borders(Borders::ALL);
        let paragraph = Paragraph::new(display_text.clone())
            .style(Style::default())
            .alignment(Alignment::Left)
            .wrap(Wrap {trim: false });
        main.render_widget(block, size);
        size.x += 1;
        size.y += 1;
        size.width -= 2;
        size.height -= 2;
        main.render_widget(paragraph, size);
        return_size = (size.height as usize) - 2;
    });
    return return_size;
}

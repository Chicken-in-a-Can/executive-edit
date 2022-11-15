/*
 * The Executive Text Editor
 * by Chicken-in-a-Can
 *
 * Hosted on Github at https://github.com/Chicken-in-a-Can/executive-edit
 *
 * Text editor written in rust for BossOS (https://github.com/Chicken-in-a-Can/the-executive-os),
 * because I got annoyed at nano, and for fun
*/
// Standard imports for various tasks
use std::{io, io::Write, thread, time::Duration, process::exit, path, fs, env};
// Rust tui and crossterm imports for rendering
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

// Main function
fn main() -> Result<(), io::Error> {
    // Read in args for file to read
    let args: Vec<String> = env::args().collect();
    // Check if args are given, fail if none are given
    if args.len() <= 1{
        println!("Specify a file to open");
        exit(1);
    }

    // Get file path and read in file
    let file_path = &args[1];
    // Creat file if doesn't exist
    if !path::Path::new(file_path).exists(){
        fs::File::create(file_path);
    }
    let mut file_path_span = Span::raw(String::from(file_path));
    let file_string = fs::read_to_string(file_path).expect("File not able to be read");
    // Read file into vector
    let mut file_vector_str: Vec<&str> = file_string.lines().collect();
    file_vector_str.push("");
    // Convert &str vector into String vector
    let mut file_vector = Vec::new();
    for i in 0..file_vector_str.len(){
        file_vector.push(file_vector_str[i].clone().to_string());
    }
    // Create new vector with line lengths
    let mut line_lengths: Vec<u16> = Vec::new();
    for i in 0..file_vector.len(){
        line_lengths.push(file_vector[i as usize].len() as u16);
    }
    let mut changes_vec: Vec<String> = Vec::new();

    // set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut span_start: usize = 0;
    let mut span_changed = (1 as u16, 1 as u16);
    let mut render_height: usize = 0;
    let mut has_saved: bool = true;
    // Create span vector from our file array
    let display_text = str_vec_to_span(file_vector.clone(), span_start.clone(), render_height.clone());

    // Render terminal
    render_height = render(&mut terminal, display_text.clone(), file_path_span.clone());
    terminal.set_cursor(1, 1);
    terminal.show_cursor();
    let mut char_input = ' ';
    let mut char_changed = false;
    let mut backspaced = false;
    let mut deleted = false;
    let mut entered = false;

    // Main loop
    loop{
        // Read key inputs
        match read().unwrap() {
            // End main loop on ctrl + q
            Event::Key(KeyEvent{
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
                ..}
            ) => break,
            Event::Key(KeyEvent{
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::CONTROL,
                ..}
            ) => bool_toggle(&mut has_saved),
            // Move on arrow key presses
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
            // move cursor to home and end of terminal based on respective keys
            Event::Key(KeyEvent{
                code: KeyCode::Home,
                modifiers: KeyModifiers::NONE,
            ..}
            ) => span_changed = cursor_home(&mut span_start),
            Event::Key(KeyEvent{
                code: KeyCode::End,
                modifiers: KeyModifiers::NONE,
            ..}
            // Backspace
            ) => span_changed = cursor_end(&mut terminal, &mut span_start, line_lengths.clone(), render_height.clone()),
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE,
                ..}
            ) => bool_toggle(&mut backspaced),
            // Delete Key
            Event::Key(KeyEvent {
                code: KeyCode::Delete,
                modifiers: KeyModifiers::NONE,
                ..}
            ) => bool_toggle(&mut deleted),
            // Enter key
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                ..}
            ) => bool_toggle(&mut entered),
            // If other random char, print it out
            Event::Key(KeyEvent{code: KeyCode::Char(event), ..}) => copy_char(&mut char_input, event, &mut char_changed),
            // if nothing, do nothing
            _ => (),
        }
        let x_pos = span_changed.0;
        let y_pos = span_changed.1;
        // If backspace was last key & not at beginning of line
        if backspaced && span_changed.0 > 1{
            file_vector[(y_pos as usize) + span_start - 1].remove(x_pos as usize - 2);
            line_lengths[(y_pos as usize) + span_start - 1] -= 1;
            span_changed.0 = span_changed.0 - 1;
        }
        // If backspace was last key && at beginning of line
        else if backspaced && span_changed.0 == 1 && span_changed.1 > 1{
            span_changed.0 = line_lengths[y_pos as usize + span_start - 2] + 1;
            span_changed.1 -= 1;
            file_vector[y_pos as usize + span_start - 2] = [file_vector[y_pos as usize + span_start - 2].clone(), file_vector[y_pos as usize + span_start - 1].clone()].join("");
            file_vector.remove(y_pos as usize + span_start - 1);
            line_lengths.remove(y_pos as usize + span_start - 1);
            line_lengths[y_pos as usize + span_start - 2] = file_vector[y_pos as usize + span_start - 2].len() as u16;
        }
        // If delete was last key & not at end of line
        if deleted && span_changed.0 < line_lengths[span_changed.1 as usize - 1]{
            file_vector[(y_pos as usize) + span_start - 1].remove(x_pos as usize - 1);
            line_lengths[(y_pos as usize) + span_start - 1] -= 1;
        }
        // If delete was last key & at end of line
        else if deleted && span_changed.0 >= line_lengths[span_changed.1 as usize - 1] && span_changed.1 < line_lengths.len() as u16{
            file_vector[y_pos as usize + span_start - 1] = [file_vector[y_pos as usize + span_start - 1].clone(), file_vector[y_pos as usize + span_start].clone()].join("");
            file_vector.remove(y_pos as usize + span_start);
            line_lengths.remove(y_pos as usize + span_start);
            line_lengths[y_pos as usize + span_start - 1] = file_vector[y_pos as usize + span_start - 1].len() as u16;
        }
        // If enter was last key
        if entered{
            file_vector.insert(y_pos as usize + span_start, "".to_owned());
            file_vector[y_pos as usize + span_start] = file_vector[y_pos as usize + span_start - 1][(x_pos as usize - 1)..].to_owned();
            file_vector[y_pos as usize + span_start - 1] = file_vector[y_pos as usize + span_start - 1][..(x_pos as usize - 1)].to_owned();
            line_lengths.insert(y_pos as usize + span_start, file_vector[y_pos as usize + span_start].len() as u16);
            span_changed.0 = 1;
            span_changed.1 = y_pos + 1;
        }
        // If random char was last key
        if char_changed{
            file_vector[(y_pos as usize) + span_start - 1].insert((x_pos as usize - 1), char_input);
            line_lengths[(y_pos as usize) + span_start - 1] += 1;
            span_changed.0 = span_changed.0 + 1;
        }
        // If was editied, say not saved
        if char_changed || backspaced || entered || deleted{
            has_saved = false;
        }
        // Create span Vector
        let display_text = str_vec_to_span(file_vector.clone(), span_start.clone(), render_height.clone());
        // Create file if opened one didn't exist
        if has_saved{
            file_path_span = Span::raw(String::from(file_path));
            save_file(file_vector.clone(), file_path);
        }
        else{
            file_path_span = Span::raw(format!("{} *", file_path));
        }
        // Re-render terminal
        render(&mut terminal, display_text.clone(), file_path_span.clone());
        let mut x_pos = span_changed.0;
        let mut y_pos = span_changed.1;
        terminal.set_cursor(x_pos, y_pos);
        terminal.show_cursor();
        // Set variables to false
        char_changed = false;
        backspaced = false;
        entered = false;
        deleted = false;
    }

    // restore Unix/Linux terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
// Set bool to false (for match cases)
fn bool_toggle(bool_input: &mut bool){
    if !*bool_input{
        *bool_input = !*bool_input;
    }
}
// Copy char to variable
fn copy_char(char_input: &mut char, char_to_copy: char, char_changed: &mut bool){
    *char_input = char_to_copy;
    *char_changed = true;
}
// Sets cursor to top of file
fn cursor_home(span_start: &mut usize) -> (u16, u16){
    *span_start = 0;
    return (1, 1);
}
// Sets cursor to bottom of file, with some blank line padding
fn cursor_end<B: tui::backend::Backend>(terminal: &mut Terminal<B>, span_start: &mut usize, line_lengths: Vec<u16>, render_height: usize) -> (u16, u16){
    let x_pos = terminal.get_cursor().unwrap().0;
    let y_pos = terminal.get_cursor().unwrap().1;
    if line_lengths.len() > render_height{
        *span_start = line_lengths.len() - render_height;
        return(1, (render_height) as u16);
    }
    else{
        return (x_pos, line_lengths.len() as u16);
    }
}
// Moves cursor, scrolls when necessary
fn cursor_move<B: tui::backend::Backend>(terminal: &mut Terminal<B>, line_lengths: Vec<u16>, mut direction: char, render_height: usize, span_start: &mut usize) -> (u16, u16){
    // Read cursor positions to variables
    let mut x_pos = terminal.get_cursor().unwrap().0;
    let mut y_pos = terminal.get_cursor().unwrap().1;
    // Scroll upwards if possible
    if y_pos <= 3 && direction == 'n' && (*span_start as usize) > 0{
        if x_pos > line_lengths[(y_pos) as usize + *span_start] + 1{
            x_pos = line_lengths[(y_pos) as usize + *span_start] + 1;
        }
        *span_start -= 1;
        return (x_pos, y_pos);
    }
    // Scroll downwards if possible
    if y_pos <= (line_lengths.len() - *span_start) as u16{
        if y_pos >= render_height as u16 && (line_lengths.len() - (*span_start as usize)) > render_height && direction == 's'{
            if x_pos > line_lengths[(y_pos) as usize + *span_start] + 1{
                x_pos = line_lengths[(y_pos) as usize + *span_start] + 1;
            }
            *span_start += 1;
            return (x_pos, y_pos);
        }
    }
    // Prevent cursor from going out of bounds
    if x_pos <= 1 && direction == 'w'{direction = 'q'}
    if y_pos <= 1 && direction == 'n'{direction = 'q'}
    // Shift variables for cursor
    match direction{
        'n' => y_pos -= 1,
        's' => y_pos += 1,
        'e' => x_pos += 1,
        'w' => x_pos -= 1,
        _ => (),
    }
    // Prevent cursor from going down further than bottom of lines
    if y_pos > (line_lengths.len() - *span_start) as u16{
        y_pos = (line_lengths.len() - *span_start) as u16;
    }
    // Keep cursor within upper bounds of x_value
    if x_pos > line_lengths[(y_pos - 1) as usize + *span_start] + 1{
        x_pos = line_lengths[(y_pos - 1) as usize + *span_start] + 1;
    }
    // Set cursor and return
    terminal.set_cursor(x_pos, y_pos);
    return (x_pos, y_pos);
}
// Converts &str vector to span vector
fn str_vec_to_span(file_vector: Vec<String>, start_y: usize, render_height: usize) -> Vec<Spans<'static>>{
    // Add first element
    let mut display_text = vec![Spans::from(file_vector[start_y].clone())];
    // Do initial conversion as well as later when under render_height number of lines after start
    if file_vector.len() <= start_y + 1 + render_height || render_height == 0{
        for i in (start_y + 1)..file_vector.len(){
            display_text.push(Spans::from(file_vector[i as usize].clone()));
        }
    }
    // Only put lines in vector if visible in render
    else{
        for i in (start_y + 1)..(start_y + 2 + render_height){
            display_text.push(Spans::from(file_vector[i as usize].clone()));
        }
    }
    return display_text
}
// Renders the terminal
fn render<B: tui::backend::Backend>(terminal: &mut Terminal<B>, display_text: Vec<Spans<'_>>, file_path_span: Span) -> usize{
    let mut render_height: usize = 0;
    terminal.draw(|main| {
        let mut size = main.size();
        // Configure border
        let block = Block::default()
            .title(file_path_span)
            .borders(Borders::ALL);
        // Configure text
        let paragraph = Paragraph::new(display_text.clone())
            .style(Style::default())
            .alignment(Alignment::Left)
            .wrap(Wrap {trim: false });
        // Render border
        main.render_widget(block, size);
        size.x += 1;
        size.y += 1;
        size.width -= 2;
        size.height -= 2;
        // Render text
        main.render_widget(paragraph, size);
        render_height = (size.height as usize) - 2;
    });
    // return height of render
    return render_height;
}
// Write file out
fn save_file(mut file_vector: Vec<String>, file_path: &str) -> std::io::Result<()>{
    let mut writer = fs::File::create(file_path)?;
    if file_vector[file_vector.len() - 1] == ""{
        file_vector.remove(file_vector.len() - 1);
    }
    for i in 0..file_vector.len(){
        writer.write_all(format!("{}\n", file_vector[i]).as_str().as_bytes())?;
    }
    Ok(())
}

use crate::commands::invoke_command;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::{cursor, event, terminal, QueueableCommand};
use std::io;
use std::io::{stdout, Stdout, Write};
use std::time::Duration;

pub struct Shell {
    stdout: Stdout,
    prompt: String,

    history: Vec<Vec<char>>,
    executed: usize,

    buffer: Vec<char>,
    cursor: usize,
    last_width: u16,
    prompt_to_cursor: u16,

    update: bool,
}

impl Shell {
    fn new() -> Shell {
        let (width, _) = terminal::size().expect("Could not query terminal width");

        Shell {
            stdout: stdout(),
            prompt: String::from("> "),

            history: Vec::new(),
            executed: 0,

            buffer: Vec::new(),
            cursor: 0,
            last_width: width,
            prompt_to_cursor: 0,

            update: false,
        }
    }
}

pub fn interactive_shell() {
    let mut shell = Shell::new();
    display_buffer(&mut shell).unwrap();
    terminal::enable_raw_mode().expect("Unable to set raw mode");
    event_loop(&mut shell).unwrap();
}

pub fn exit_interactive(exit_code: i32) -> io::Result<i32> {
    terminal::disable_raw_mode()?;
    std::process::exit(exit_code);
}

pub fn prompt_clear(shell: &mut Shell, clear_type: terminal::ClearType) -> io::Result<i32> {
    shell.stdout.queue(cursor::MoveTo(0, 0))?;
    shell.stdout.queue(terminal::Clear(clear_type))?;
    shell.prompt_to_cursor = 0;
    shell.update = true;
    Ok(0)
}

fn event_loop(shell: &mut Shell) -> io::Result<()> {
    let default_poll_duration = Duration::from_millis(100);
    let mut poll_duration = default_poll_duration;

    loop {
        if event::poll(poll_duration)? {
            while {
                match event::read()? {
                    Event::Key(key) => handle_key_event(key, shell)?,
                    Event::Resize(_, _) => shell.update = true,
                    _ => (),
                }
                event::poll(Duration::from_millis(0))?
            } {}
        }

        poll_duration = default_poll_duration;

        if shell.update {
            display_buffer(shell)?;
        }

        match invoke_prompt(shell) {
            Ok(Some(_)) => {
                display_buffer(shell)?;
                poll_duration = Duration::from_millis(0);
            }
            Err(e) => return Err(e),
            _ => {}
        }
    }
}

fn handle_key_event(key: KeyEvent, shell: &mut Shell) -> io::Result<()> {
    match (key.modifiers, key.code) {
        (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(char)) => {
            prompt_input(shell, char)
        }
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => prompt_erase(shell),
        (KeyModifiers::CONTROL, KeyCode::Char('d')) => { exit_interactive(0)?; },
        (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
            prompt_clear(shell, terminal::ClearType::All)?;
        }
        (KeyModifiers::NONE, KeyCode::Left)      => prompt_shift_cursor(shell, -1),
        (KeyModifiers::NONE, KeyCode::Right)     => prompt_shift_cursor(shell, 1),
        (KeyModifiers::NONE, KeyCode::Backspace) => prompt_delete(shell, -1),
        (KeyModifiers::NONE, KeyCode::Delete)    => prompt_delete(shell, 0),
        (KeyModifiers::NONE, KeyCode::Enter)     => prompt_enter(shell),
        (_, KeyCode::Home)                       => prompt_set_cursor(shell, 0),
        (_, KeyCode::End)                        => prompt_set_cursor(shell, shell.buffer.len()),
        _ => (),
    }
    Ok(())
}

fn invoke_prompt(shell: &mut Shell) -> io::Result<Option<i32>> {
    match shell.history.get(shell.executed) {
        None => Ok(None),
        Some(buffer) => {
            shell.executed += 1;
            shell.stdout.queue(cursor::MoveToColumn(0))?;
            println!();

            let command = buffer.iter().collect::<String>();
            let args = shlex::split(command.as_str()).expect("Failed to split command");
            Ok(Some(invoke_command(Some(shell), &args)?))
        }
    }
}

fn display_buffer(shell: &mut Shell) -> io::Result<()> {
    let (width, _) = terminal::size()?;
    let (_, row) = cursor::position()?;

    let (buffer, cursor) = match shell.history.get(shell.executed) {
        Some(buffer) => (buffer, buffer.len()),
        None => (&shell.buffer, shell.cursor),
    };

    shell.stdout.queue(cursor::MoveTo(0, row - shell.prompt_to_cursor))?;
    shell.stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;
    shell.stdout.queue(terminal::Clear(terminal::ClearType::CurrentLine))?;

    let (formatted, cursor_col, cursor_row) = format_buffer(&buffer, cursor, width);
    print!("{}", formatted);
    shell.stdout.queue(cursor::MoveTo(cursor_col, row - shell.prompt_to_cursor + cursor_row))?;
    shell.stdout.flush()?;

    shell.prompt_to_cursor = cursor_row;
    shell.last_width = width;
    shell.update = false;
    Ok(())
}

fn format_buffer(buffer: &Vec<char>, cursor: usize, width: u16) -> (String, u16, u16) {
    const PROMPT: &str = "> ";
    let (mut col, mut row) = (PROMPT.len() as u16, 0);

    let mut formatted = String::from(PROMPT);

    for (i, char) in buffer.iter().enumerate() {
        formatted.push(*char);
        if i < cursor {
            if col == width - 1 {
                row += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
    }

    if cursor == buffer.len() {
        formatted.push(' ');
    }

    (formatted, col, row)
}

fn prompt_set_cursor(shell: &mut Shell, pos: usize) {
    if pos != shell.cursor {
        shell.cursor = pos;
        shell.update = true;
    }
}

fn prompt_shift_cursor(shell: &mut Shell, delta: isize) {
    match shell.cursor.checked_add_signed(delta) {
        Some(pos) if pos <= shell.buffer.len() => {
            shell.cursor = pos;
            shell.update = true;
        }
        _ => {}
    }
}

fn prompt_input(shell: &mut Shell, char: char) {
    shell.buffer.insert(shell.cursor, char);
    shell.cursor += 1;
    shell.update = true;
}

fn prompt_delete(shell: &mut Shell, delta: isize) {
    match shell.cursor.checked_add_signed(delta) {
        Some(pos) if pos < shell.buffer.len() => {
            shell.buffer.remove(pos);
            shell.cursor = pos;
            shell.update = true;
        }
        _ => {}
    }
}

fn prompt_erase(shell: &mut Shell) {
    shell.buffer.clear();
    shell.cursor = 0;
    shell.update = true;
}

fn prompt_enter(shell: &mut Shell) {
    if shell.buffer.last() == Some(&'\\') {
        todo!("Multiline commands");
    } else {
        shell.history.push(shell.buffer.clone());
        shell.buffer.clear();
    }

    shell.cursor = 0;
    shell.update = true;
}

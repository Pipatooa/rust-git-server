use std::ffi::OsStr;
use std::io;
use std::io::{stdout, Stdout, Write};
use std::panic::UnwindSafe;
use std::process::{Command, ExitStatus};
use std::time::Duration;
use crossterm::{cursor, event, terminal, QueueableCommand};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

pub fn invoke_command<I, S>(command: &str, args: I) -> io::Result<ExitStatus>
where I: IntoIterator<Item=S>, S: AsRef<OsStr> {
    let mut command = Command::new(command);
    command.args(args);

    match command.spawn() {
        Ok(mut child) => child.wait(),
        Err(e) => Err(e)
    }
}

struct Shell {
    stdout: Stdout,
    prompt: String,

    history: Vec<Vec<char>>,
    executed: usize,

    buffer: Vec<char>,
    cursor: usize,

    update: bool
}

impl Shell {
    fn new() -> Shell {
        Shell {
            stdout: stdout(),
            prompt: String::from("> "),

            history: Vec::new(),
            executed: 0,

            buffer: Vec::new(),
            cursor: 0,

            update: false,
        }
    }
}

impl UnwindSafe for Shell {}

pub fn interactive_shell() {
    let mut shell = Shell::new();
    display_buffer(&mut shell).unwrap();
    terminal::enable_raw_mode().ok();
    event_loop(&mut shell).unwrap();
}

fn event_loop(shell: &mut Shell) -> Result<(), io::Error> {
    loop {
        if event::poll(Duration::from_millis(100))? {
            while {
                match event::read()? {
                    Event::Key(key) => handle_key_event(key, shell)?,
                    _ => ()
                }
                event::poll(Duration::from_millis(0))?
            } {}
        }

        if shell.update {
            display_buffer(shell)?;
        }
    }
}

fn display_buffer(shell: &mut Shell) -> Result<(), io::Error>{
    shell.stdout.queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
    shell.stdout.queue(cursor::MoveToColumn(0))?;
    print!("> {}", shell.buffer.iter().collect::<String>());
    shell.stdout.queue(cursor::MoveToColumn((shell.cursor + 2) as u16))?;

    shell.stdout.flush()?;
    shell.update = false;
    Ok(())
}

fn handle_key_event(key: KeyEvent, shell: &mut Shell) -> Result<(), io::Error> {
    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('d')) => std::process::exit(0),
        (_, KeyCode::Home) => { shell.cursor = 0; shell.update = true; }
        (_, KeyCode::End) => { shell.cursor = shell.buffer.len(); shell.update = true; }
        (KeyModifiers::NONE, KeyCode::Left)  => {
            if shell.cursor > 0 {
                shell.cursor -= 1;
                shell.update = true;
            }
        },
        (KeyModifiers::NONE, KeyCode::Right) => {
            if shell.cursor < shell.buffer.len() {
                shell.cursor += 1;
                shell.update = true;
            }
        },
        (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(char)) => {
            shell.buffer.insert(shell.cursor, char);
            shell.cursor += 1;
            shell.update = true;
        },
        (KeyModifiers::NONE, KeyCode::Backspace) => {
            if shell.cursor > 0 {
                shell.buffer.remove(shell.cursor - 1);
                shell.cursor -= 1;
                shell.update = true;
            }
        },
        (KeyModifiers::NONE, KeyCode::Delete) => {
            if shell.cursor < shell.buffer.len() {
                shell.buffer.remove(shell.cursor);
                shell.update = true;
            }
        },
        (KeyModifiers::NONE, KeyCode::Enter) => {
            if shell.buffer.last() == Some(&'\\') {
                todo!("Multiline commands");
            } else {
                shell.history.push(shell.buffer.clone());
                shell.buffer.clear();
            }

            shell.cursor = 0;
            shell.update = true;
        }
        _ => ()
    }
    Ok(())
}

fn invoke_prompt(_buffer: &Vec<char>) -> io::Result<ExitStatus> {
    todo!()
}

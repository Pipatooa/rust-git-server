use std::ffi::OsStr;
use std::io;
use std::process::{Command, ExitStatus};

pub fn invoke_command<I, S>(command: &str, args: I) -> io::Result<Option<ExitStatus>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(command);
    command.args(args);

    match command.spawn() {
        Ok(mut child) => Ok(Some(child.wait()?)),
        Err(e) => Err(e),
    }
}

use shim::io;
use shim::path::{Path, PathBuf};

use stack_vec::StackVec;

use pi::atags::Atags;

use fat32::traits::FileSystem;
use fat32::traits::{Dir, Entry};

use crate::console::{kprint, kprintln, CONSOLE};
use crate::ALLOCATOR;
use crate::FILESYSTEM;

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs,
}

/// A structure representing a single shell command.
struct Command<'a> {
    args: StackVec<'a, &'a str>,
}

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'a str, buf: &'a mut [&'a str]) -> Result<Command<'a>, Error> {
        let mut args = StackVec::new(buf);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg).map_err(|_| Error::TooManyArgs)?;
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        self.args[0]
    }

    fn extract_path(&self) -> Option<PathBuf> {
        if self.args.len() < 2 {
            return None
        }
        if self.args[self.args.len() - 1] == "-a" {
            return None
        }
        Some(PathBuf::from(self.args[self.args.len() - 1]))
    }

    fn ls(&self, working_dir: &PathBuf) -> Result<(), ()> {
        use alloc::vec::Vec;
        use alloc::string::String;
        use fat32::traits::Timestamp;

        let mut show_hidden = false;

        for arg in &self.args {
            if arg == &"-a" {
                show_hidden = true;
            }
        }

        let mut path = working_dir.clone();
        let arg_path = self.extract_path();
        if let Some(p) = arg_path {
            path.push(p);
        }


        let mut entries = FILESYSTEM
            .open_dir(path.clone())
            .expect("directory")
            .entries()
            .expect("entries interator")
            .collect::<Vec<_>>();
        entries.sort_by(|a, b| a.name().cmp(b.name()));
        for entry in &entries {
            let atime = entry.metadata().atime;
            let hidden = entry.metadata().attr.hidden() && !show_hidden;
            if hidden {
                continue;
            }
            let is_dir_str = if entry.is_dir() { &"d" } else { &"" };
            let size_str = if let Some(f) = entry.as_file() { f.size } else { 0 };
            kprintln!("{0: <1} {1:02}/{2:02}/{3} {4:02}:{5:02}:{6:02} {7: <10} {8: <10}",
                      is_dir_str,
                      atime.month(),
                      atime.day(),
                      atime.year(),
                      atime.hour(),
                      atime.minute(),
                      atime.second(),
                      size_str,
                      entry.name())

        }
        //kprintln!("'/': {:?}", entries);
        Ok(())

    }

    fn exec(&self, working_dir: &mut PathBuf) -> Result<(), ()> {
        match self.path() {
            "echo" => {
                for arg in self.args[1..self.args.len() - 1].iter() {
                    kprint!("{} ", arg);
                }
                kprintln!("{}", self.args[self.args.len() - 1]);
            },
            "ls" => {self.ls(&working_dir);},
            _ => kprintln!("{}: command not found", self.path()),
        }
        Ok(())
    }
}

fn readline(buf: &mut [u8]) -> &str {
    use core::str;
    let mut line: StackVec<u8> = StackVec::new(buf);

    loop {
        match CONSOLE.lock().read_byte() {
            0x08 | 0x7F if !line.is_empty() => {
                kprint!("\u{8} \u{8}");
                line.pop();
            },
            b'\r' | b'\n' => {
                kprintln!();
                break;
            },
            b @ b' ' ..=b'~' if !line.is_full() => {
                kprint!("{}", b as char);
                line.push(b);
            },
            _ => kprint!("\u{7}"),
        }
    }
    str::from_utf8(line.into_slice()).unwrap()
}

/// Starts a shell using `prefix` as the prefix for each line. This function
/// never returns.
pub fn shell(prefix: &str) -> ! {
    let mut working_dir = PathBuf::from("/");
    loop {
        kprint!("{}", prefix);
        match Command::parse(readline(&mut [0u8; 512]), &mut [""; 64]) {
            Ok(c) => c.exec(&mut working_dir).unwrap(),
            Err(Error::TooManyArgs) => kprintln!("error: too many args"),
            Err(_) => (),
        }
    }
}

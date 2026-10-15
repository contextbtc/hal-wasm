//! The subset of submodules/hal/src/bin/hal/util.rs that the commands use.

use std::borrow::Cow;
use std::io::Read;
use std::{fmt, io};

use crate::prelude::*;

/// Get the named argument from the CLI arguments or try read from stdin if not provided.
pub fn arg_or_stdin<'a>(args: &'a clap::ArgMatches<'a>, arg: &str) -> Cow<'a, str> {
    if let Some(s) = args.value_of(arg) {
        s.into()
    } else {
        // Read from stdin.
        let mut input = Vec::new();
        let stdin = io::stdin();
        let mut stdin_lock = stdin.lock();
        let _ = stdin_lock.read_to_end(&mut input);
        while stdin_lock.read_to_end(&mut input).unwrap_or(0) > 0 {}
        if input.is_empty() {
            exit!("no '{}' argument given", arg);
        }
        String::from_utf8(input)
            .need(&format!("invalid utf8 on stdin for '{}'", arg))
            .trim()
            .to_owned()
            .into()
    }
}

pub fn more_than_one(bools: &[bool]) -> bool {
    let mut one = false;
    for b in bools {
        if *b && one {
            return true;
        }
        one |= *b;
    }
    false
}

pub trait ResultExt<T, E: fmt::Display>: Into<Result<T, E>> {
    #[track_caller]
    fn need(self, msg: &str) -> T {
        match self.into() {
            Ok(t) => t,
            Err(e) => exit!("{}: {}", msg, e),
        }
    }
}

impl<T, E: fmt::Display> ResultExt<T, E> for std::result::Result<T, E> {}

pub trait OptionExt<T>: Into<Option<T>> {
    #[track_caller]
    fn need(self, msg: &str) -> T {
        match self.into() {
            Some(t) => t,
            None => exit!("{}", msg),
        }
    }
}

impl<T> OptionExt<T> for std::option::Option<T> {}

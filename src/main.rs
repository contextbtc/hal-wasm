//! Entry point for hal on WebAssembly (WASI).
//!
//! This mirrors submodules/hal/src/bin/hal/main.rs, minus external `hal-*` subcommands,
//! which need PATH lookups and process spawning that WASI doesn't provide.
//! The commands themselves are compiled unmodified from the hal submodule.

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

use std::{env, panic, process};

#[path = "../submodules/hal/src/bin/hal/args.rs"]
pub mod args;
#[path = "../submodules/hal/src/bin/hal/cmd/mod.rs"]
pub mod cmd;
pub mod util;

pub mod prelude {
    pub use super::exit;
    pub use crate::args::{ArgMatchesExt, FlexiblePubkey};
    pub use crate::util::{OptionExt, ResultExt};
    pub use crate::{args, cmd, util};
    pub use hal::SECP;
}

use crate::prelude::*;

#[macro_export]
macro_rules! exit {
	($($arg:tt)*) => {{
		if $crate::init_app().get_matches().is_present("verbose") {
			let msg = format!($($arg)*);
			eprintln!("{}", msg);
			panic!("{}", msg);
		} else {
			eprintln!($($arg)*);
			std::process::exit(1);
		}
	}}
}

/// Setup logging with the given log level.
fn setup_logger(lvl: log::LevelFilter) {
    fern::Dispatch::new()
        .format(|out, message, _record| out.finish(format_args!("{}", message)))
        .level(lvl)
        .chain(std::io::stderr())
        .apply()
        .need("error setting up logger");
}

/// Create the main app object.
fn init_app() -> clap::App<'static, 'static> {
    clap::App::new("hal")
        .version(env!("HAL_VERSION"))
        .author("Steven Roose <steven@stevenroose.org>")
        .about("hal - the Bitcoin companion")
        .settings(&[
            clap::AppSettings::GlobalVersion,
            clap::AppSettings::UnifiedHelpMessage,
            clap::AppSettings::VersionlessSubcommands,
            clap::AppSettings::AllowExternalSubcommands,
            clap::AppSettings::DisableHelpSubcommand,
            clap::AppSettings::AllArgsOverrideSelf,
            clap::AppSettings::SubcommandRequiredElseHelp,
        ])
        .subcommands(cmd::subcommands())
        .arg(
            args::flag("verbose", "Print verbose logging output to stderr")
                .short("v")
                .global(true),
        )
        .args(&args::opts_networks())
        .arg(&args::opt_yaml())
}

fn main() {
    // Apply a custom panic hook to print a more user-friendly message
    // in case the execution fails.
    // We skip this for people that are interested in the panic message.
    if env::var("RUST_BACKTRACE").unwrap_or(String::new()) != "1" {
        panic::set_hook(Box::new(|info| {
            let message = if let Some(m) = info.payload().downcast_ref::<String>() {
                m
            } else if let Some(m) = info.payload().downcast_ref::<&str>() {
                m
            } else {
                "No error message provided"
            };
            eprintln!("Execution failed: {}", message);
            process::exit(1);
        }));
    }

    let args = init_app().get_matches();

    // Enable logging in verbose mode.
    match args.is_present("verbose") {
        true => setup_logger(log::LevelFilter::Trace),
        false => setup_logger(log::LevelFilter::Warn),
    }

    match args.subcommand() {
        ("address", Some(ref m)) => cmd::address::execute(&m),
        ("ark", Some(ref m)) => cmd::ark::execute(&m),
        ("bech32", Some(ref m)) => cmd::bech32::execute(&m),
        ("bip32", Some(ref m)) => cmd::bip32::execute(&m),
        ("bip39", Some(ref m)) => cmd::bip39::execute(&m),
        ("block", Some(ref m)) => cmd::block::execute(&m),
        ("hash", Some(ref m)) => cmd::hash::execute(&m),
        ("key", Some(ref m)) => cmd::key::execute(&m),
        ("ln", Some(ref m)) => cmd::ln::execute(&m),
        ("merkle", Some(ref m)) => cmd::merkle::execute(&m),
        ("message", Some(ref m)) => cmd::message::execute(&m),
        ("miniscript", Some(ref m)) => cmd::miniscript::execute(&m),
        ("psbt", Some(ref m)) => cmd::psbt::execute(&m),
        ("random", Some(ref m)) => cmd::random::execute(&m),
        ("script", Some(ref m)) => cmd::script::execute(&m),
        ("tx", Some(ref m)) => cmd::tx::execute(&m),
        (cmd, _) => exit!("no such subcommand: `{}`", cmd),
    }
}

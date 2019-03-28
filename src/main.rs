//! mdbook-bookimport is a pre-processor for [mdbook]'s that helps you avoid link rot
//! when importing parts of other files into your mdbook.
//!
//! [mdbook]: https://github.com/rust-lang-nursery/mdBook

#![deny(missing_docs, warnings)]

#[macro_use]
extern crate log;

use env_logger::Builder;

use chrono::Local;
use log::LevelFilter;
use std::{env, process};

use clap::{App, Arg, ArgMatches, SubCommand};
use mdbook::{
    errors::Error,
    preprocess::{CmdPreprocessor, Preprocessor},
};
use mdbook_bookimport::Bookimport;

fn main() {
    init_logging();

    let matches = make_cli().get_matches();

    let bookimport = Bookimport {};

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(&bookimport, sub_args);
    } else {
        if let Err(e) = handle_preprocessing(&bookimport) {
            error!("{}", e);
            process::exit(1);
        }
    }
}

// Used by mdbook to determine whether or not our binary can be used as a pre-processor
fn make_cli() -> App<'static, 'static> {
    App::new("mdbook-bookimport")
        .about("Import code/text from other files into your mdbook - without the link rot.")
        .subcommand(
            SubCommand::with_name("supports")
                .arg(Arg::with_name("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

// Used by mdbook to determine whether or not our binary can be used as a pre-processor
fn handle_supports(bookimport: &dyn Preprocessor, sub_args: &ArgMatches) -> ! {
    let renderer = sub_args.value_of("renderer").expect("Required argument");
    let supported = bookimport.supports_renderer(&renderer);

    // Signal whether the renderer is supported by exiting with 1 or 0.
    if supported {
        process::exit(0);
    } else {
        process::exit(1);
    }
}

// Run our preprocessor to replace #bookimport's in every chapter in an mdbook
fn handle_preprocessing(bookimport: &dyn Preprocessor) -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(::std::io::stdin())?;

    let book_after_bookimport = bookimport.run(&ctx, book)?;

    serde_json::to_writer(::std::io::stdout(), &book_after_bookimport)?;

    Ok(())
}

fn init_logging() {
    use std::io::Write;
    let mut builder = Builder::new();

    builder.format(|formatter, record| {
        writeln!(
            formatter,
            "{} [{}] ({}): {}",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.target(),
            record.args()
        )
    });

    if let Ok(var) = env::var("RUST_LOG") {
        builder.parse_filters(&var);
    } else {
        // if no RUST_LOG provided, default to logging at the Info level
        builder.filter(None, LevelFilter::Info);
        // Filter extraneous html5ever not-implemented messages
        builder.filter(Some("html5ever"), LevelFilter::Error);
    }

    builder.init();
}

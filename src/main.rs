#[macro_use]
extern crate log;

use env_logger::Builder;

use chrono::Local;
use log::LevelFilter;
use std::{env, process};

use clap::{App, Arg, ArgMatches, SubCommand};
use mdbook::{
    book::{Book, BookItem},
    errors::{Error, Result as MdResult},
    preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext},
};
use mdbook_superimport::Superimport;
use std::io::{stdin, Read};

pub fn make_app() -> App<'static, 'static> {
    App::new("mdbook-superimport")
        .about("A mdbook preprocessor which does precisely nothing")
        .subcommand(
            SubCommand::with_name("supports")
                .arg(Arg::with_name("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() {
    init_logging();

    let matches = make_app().get_matches();

    let superimport = Superimport {};

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(&superimport, sub_args);
    } else {
        if let Err(e) = handle_preprocessing(&superimport) {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

fn handle_supports(superimport: &dyn Preprocessor, sub_args: &ArgMatches) -> ! {
    let renderer = sub_args.value_of("renderer").expect("Required argument");
    let supported = superimport.supports_renderer(&renderer);

    // Signal whether the renderer is supported by exiting with 1 or 0.
    if supported {
        process::exit(0);
    } else {
        process::exit(1);
    }
}

fn handle_preprocessing(superimport: &dyn Preprocessor) -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(::std::io::stdin())?;

    let book_after_superimport = superimport.run(&ctx, book)?;

    serde_json::to_writer(::std::io::stdout(), &book_after_superimport)?;

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

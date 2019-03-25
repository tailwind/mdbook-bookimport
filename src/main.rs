use env_logger::Builder;

use std::env;
use log::LevelFilter;
use chrono::Local;

use mdbook::{
    book::{Book, BookItem},
    errors::{Error, Result as MdResult},
    preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext},
};
use mdbook_superimport::Superimport;

fn main() -> mdbook::errors::Result<()> {
    init_logging();

    let (ctx, book) = CmdPreprocessor::parse_input(::std::io::stdin())?;

    let superimport = Superimport {};

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

use failure::Fail;
use log::*;
use mdbook::book::{Book, Chapter};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use mdbook::BookItem;
use std::path::PathBuf;

/// The pre-processor that powers the mdbook-superimport plugin
pub struct Superimport;

impl Preprocessor for Superimport {
    fn name(&self) -> &str {
        "mdbook-superimport"
    }

    fn run(
        &self,
        ctx: &PreprocessorContext,
        mut book: Book,
    ) -> Result<Book, mdbook::errors::Error> {
        for section in book.sections.iter_mut() {
            if let BookItem::Chapter(ref mut chapter) = section {
                process_chapter(chapter)?;
            }
        }

        Ok(book)
    }
}

fn process_chapter(chapter: &mut Chapter) -> mdbook::errors::Result<()> {
    Ok(())
}

/// # Example
///
/// If you look at book/src/introduction.md you'll see this super import:
///
/// ```md,ignore
/// {{#simport ../book.toml@super-section }}
/// ```
///
/// Which refers to this part of our book/book.toml
///
/// ```toml,ignore
/// # @simport start super-section
/// [preprocessor.superimport]
/// // ...
/// # @simport end super-section
/// ```
///
/// The doc comments on the struct fields refer to this simport
#[derive(Debug, PartialEq)]
struct Simport<'a> {
    /// The book chapter that this #simport was found in
    ///
    /// introduction.md
    host_chapter: &'a Chapter,
    /// The filepath relative to the chapter
    ///
    /// ../book.toml
    file: PathBuf,
    /// Tags after the characters after an `@` symbol. When importing from a file
    /// Superimport will pull all text before and after the `@tag`
    ///
    /// Some(super-section)
    tag: Option<&'a str>,
}

impl<'a> Simport<'a> {
    fn parse_chapter<'c>(chapter: &'c Chapter) -> Vec<Simport<'c>> {
        let simports = chapter
            .content
            .lines()
            .filter(|line| line.contains("#simport"))
            // [ "{{#simport ./fixture.css@cool-css }}" ] -> Simport { ... }
            .map(|simport_line| {
                let mut after_simport = simport_line.split("#simport");
                after_simport.next().unwrap();
                // ./fixture.css@cool-css
                let after_simport = after_simport.next().unwrap();

                // [./fixture.css, cool-css }}
                let mut pieces = after_simport.split("@");

                let file = pieces.next().unwrap().trim();

                // cool-css }}
                let mut remaining = pieces.next().unwrap().split(" ");
                // cool-css
                let tag = match remaining.next() {
                    Some(tag) => Some(tag.trim()),
                    None => None
                };

                Simport {
                    host_chapter: &chapter,
                    file: file.into(),
                    tag,
                }
            })
            .collect();

        simports
    }
}

#[derive(Debug, Fail)]
enum TagError {
    #[fail(display = "Could not find `@simport start {}`", tag)]
    MissingStartTag { tag: String },
}

fn read_content_between_tags<'a>(content: &'a str, tag: &str) -> Result<&'a str, TagError> {
    let content_between_tags = "";

    Ok(content_between_tags)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simports_from_chapter() {
        let tag_import_chapter = make_tag_import_chapter();

        let simports = Simport::parse_chapter(&tag_import_chapter);

        let expected_simports = vec![Simport {
            host_chapter: &tag_import_chapter,
            file: "./fixture.css".into(),
            tag: Some("cool-css"),
        }];

        assert_eq!(simports, expected_simports);
    }

    // Create a chapter to represent our tag-import test case in the /book
    // directory in this repo.
    fn make_tag_import_chapter() -> Chapter {
        let chapter = "book/src/test-cases/tag-import/README.md";

        let tag_import_chapter = Chapter::new(
            "Tag Import",
            include_str!("../book/src/test-cases/tag-import/README.md").to_string(),
            &format!("{}/{}", env!("CARGO_MANIFEST_DIR"), chapter),
            vec![],
        );

        tag_import_chapter
    }
}

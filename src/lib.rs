use failure::Fail;
use log::*;
use mdbook::book::{Book, Chapter};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use mdbook::BookItem;
use std::path::{Path, PathBuf};

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
    let mut content = chapter.content.clone();

    let simports = Simport::parse_chapter(chapter);

    for simport in simports {
        let new_content = match simport.read_content_between_tags() {
            Ok(new_content) => new_content,
            Err(err) => panic!("{:#?}", err) // FIXME: Return failure with `?`
        };

        // TODO: Replace the line within the content with the new_content
        // content = new_content
    }

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
    /// The line in the file that this simport occurs on
    ///
    /// 1
    line: usize,
}

impl<'a> Simport<'a> {
    fn parse_chapter<'c>(chapter: &'c Chapter) -> Vec<Simport<'c>> {
        let simports = chapter
            .content
            .lines()
            .enumerate()
            .filter(|(idx, line)| line.contains("#simport"))
            // [ "{{#simport ./fixture.css@cool-css }}" ] -> Simport { ... }
            .map(|(idx, simport_line)| {
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
                    None => None,
                };

                Simport {
                    host_chapter: &chapter,
                    file: file.into(),
                    tag,
                    line: idx + 1
                }
            })
            .collect();

        simports
    }
}

#[derive(Debug, Fail, PartialEq)]
enum TagError {
    #[fail(display = "Could not find `@simport start {}`", tag)]
    MissingStartTag { tag: String },
}

impl<'a> Simport<'a> {
    // TODO: Clean up - don't need 3 iterations through the file.. Do it in for loop.
    fn read_content_between_tags(&self) -> Result<String, TagError> {
        let tag = self.tag.unwrap();

        let chapter_dir = self.host_chapter.path.parent().unwrap();
        let path = Path::join(Path::new(&chapter_dir), &self.file);
        let content = String::from_utf8(::std::fs::read(&path).unwrap()).unwrap();

        let start_line = content
            .lines()
            .enumerate()
            .filter(|(_line_num, line_content)| line_content.contains("@simport start"))
            .map(|(line_num, _)| line_num)
            .next();

        let end_line = content
            .lines()
            .enumerate()
            .filter(|(_line_num, line_content)| line_content.contains("@simport end"))
            .map(|(line_num, _)| line_num)
            .next();

        // FIXME: Return TagError if there is no start or end tag in the file
        let start_line = start_line.unwrap();
        let end_line = end_line.unwrap();

        let content_between_tags: Vec<String> = content
            .lines()
            .enumerate()
            .filter(|(line_num, line_content)| *line_num > start_line && *line_num < end_line)
            .map(|(line_num, line_content)| line_content.to_string())
            .collect();
        let content_between_tags = content_between_tags.join("\n");

        Ok(content_between_tags)
    }
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
            line: 4
        }];

        assert_eq!(simports, expected_simports);
    }

    #[test]
    fn content_between_tags() {
        let tag_import_chapter = make_tag_import_chapter();

        let simport = &Simport::parse_chapter(&tag_import_chapter)[0];

        let content_between_tags = simport.read_content_between_tags();

        let expected_content = r#".this-will-be-included {
  display: block;
}"#;

        assert_eq!(content_between_tags.unwrap(), expected_content);
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

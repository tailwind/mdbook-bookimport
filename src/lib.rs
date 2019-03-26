use failure::Fail;
use lazy_static::lazy_static;
use log::*;
use mdbook::book::{Book, Chapter};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use mdbook::BookItem;
use regex::Regex;
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
        debug!("Running `run` method in superimport Preprocessor trait impl");

        let book_src_dir = ctx.root.join(&ctx.config.book.src);

        for section in book.sections.iter_mut() {
            process_chapter(section, &book_src_dir);
        }

        Ok(book)
    }
}

fn process_chapter(book_item: &mut BookItem, book_src_dir: &PathBuf) -> mdbook::errors::Result<()> {
    // FIXME: Make process_chapter method take the BookItem
    if let BookItem::Chapter(ref mut chapter) = book_item {
        debug!("Processing chapter {}", chapter.name);

        let chapter_dir = chapter
            .path
            .parent()
            .map(|dir| book_src_dir.join(dir))
            .expect("All book items have a parent");

        let mut content = chapter.content.clone();

        let simports = SuperImport::parse_chapter(chapter);

        // Iterate backwards through the simports so that we start by replacing the imports
        // that are lower in the file first.
        //
        // This ensures that as we replace simports we aren't throwing off the start and end
        // indices of other simports.
        for simport in simports.iter().rev() {
            // TODO: BREADCRUMB If the full_simport_text begins with a `\` just continue.
            // Write a test case for this by importing from our test-cases directory

            let new_content = match simport.read_content_between_tags(&chapter_dir) {
                Ok(new_content) => new_content,
                Err(err) => panic!("{:#?}", err), // FIXME: Return failure with `?`
            };

            content = content.replace(simport.full_simport_text, &new_content);
        }

        chapter.content = content;

        for sub_item in chapter.sub_items.iter_mut() {
            process_chapter(sub_item, book_src_dir)?;
        }
    }

    Ok(())
}

/// # Example
///
/// If you look at book/src/introduction.md you'll see this super import:
///
/// ```md,ignore
/// {{#superimport ../book.toml@super-section }}
/// ```
///
/// Which refers to this part of our book/book.toml
///
/// ```toml,ignore
/// # @superimport start super-section
/// [preprocessor.superimport]
/// // ...
/// # @superimport end super-section
/// ```
///
/// The doc comments on the struct fields refer to this superimport
#[derive(Debug, PartialEq)]
struct SuperImport<'a> {
    /// The book chapter that this #superimport was found in
    ///
    /// introduction.md
    host_chapter: &'a Chapter,
    /// The filepath relative to the chapter
    ///
    /// ../book.toml
    file: PathBuf,
    /// The text of this superimport in the host_chapter
    ///
    /// {{ #superimport some-file.txt@some-tag }}
    full_simport_text: &'a str,
    /// Tags after the characters after an `@` symbol. When importing from a file
    /// Superimport will pull all text before and after the `@tag`
    ///
    /// Some(super-section)
    tag: &'a str,
    /// Where in the chapter's bytes does this superimport start?
    start: usize,
    /// Where in the chapter's bytes does this superimport end?
    end: usize,
}

// Wrapping in lazy_static ensures that our regex is only compiled once
lazy_static! {
  /// The regex that finds superimports such as -> `{{ #superimport some-file.txt@some-tag }}`
  static ref RE: Regex = Regex::new(
  r"(?x)                        # (?x) means insignificant whitespace mode
                                # allows us to put comments and space things out.

    \\\{\{\#.*\}\}                # escaped import such as `\{{ #superimport some-file.txt@some-tag }}`

  |                               # OR

                                  # Non escaped import -> `{{ #superimport some-file.txt@some-tag }}`
    \{\{\s*                         # opening braces and whitespace
    \#superimport                       # #superimport
    \s+                             # separating whitespace
    (?P<file>[a-zA-Z0-9\s_.\-/\\]+) # some-file.txt
    @                               # @ symbol that denotes the name of a tag
    (?P<tag>[a-zA-Z0-9_.\-]+)       # some-tag (alphanumeric underscores and dashes)
    \s*\}\}                         # whitespace and closing braces
  "
  ).unwrap();
}

impl<'a> SuperImport<'a> {
    fn parse_chapter(chapter: &Chapter) -> Vec<SuperImport> {
        let mut simports = vec![];

        let matches = RE.captures_iter(chapter.content.as_str());

        for capture_match in matches {
            // {{#superimport ./fixture.css@cool-css }}
            //    OR
            // \{{#superimport ./fixture.css@cool-css }}
            let full_capture = capture_match.get(0).unwrap();

            let full_simport_text = &chapter.content[full_capture.start()..full_capture.end()];

            // NOTE: The backslash means that this import was escaped by the author, so
            // we don't want to replace it.
            // \{{#superimport ./fixture.css@cool-css }}
            if full_simport_text.starts_with(r"\") {
                continue;
            }

            let file = capture_match["file"].into();
            let tag = capture_match.get(2).unwrap();

            let simport = SuperImport {
                host_chapter: chapter,
                file,
                full_simport_text,
                tag: &chapter.content[tag.start()..tag.end()],
                start: full_capture.start(),
                end: full_capture.end(),
            };

            simports.push(simport);
        }

        simports
    }
}

#[derive(Debug, Fail, PartialEq)]
enum TagError {
    #[fail(display = "Could not find `@superimport start {}`", tag)]
    MissingStartTag { tag: String },
}

impl<'a> SuperImport<'a> {
    // TODO: Clean up - don't need 3 iterations through the file.. Do it in for loop.
    fn read_content_between_tags(&self, chapter_dir: &PathBuf) -> Result<String, TagError> {
        debug!(
            r#"Reading content in chapter "{}" for superimport "{:#?}" "#,
            self.host_chapter.name, self.full_simport_text
        );

        let tag = self.tag;

        let path = Path::join(&chapter_dir, &self.file);

        let content = String::from_utf8(::std::fs::read(&path).unwrap()).unwrap();

        let start_line = content
            .lines()
            .enumerate()
            .filter(|(_line_num, line_content)| line_content.contains("@superimport start"))
            .map(|(line_num, _)| line_num)
            .next();

        let end_line = content
            .lines()
            .enumerate()
            .filter(|(_line_num, line_content)| line_content.contains("@superimport end"))
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

        let simports = SuperImport::parse_chapter(&tag_import_chapter);

        let expected_simports = vec![SuperImport {
            host_chapter: &tag_import_chapter,
            file: "./fixture.css".into(),
            full_simport_text: "{{#superimport ./fixture.css@cool-css }}",
            tag: "cool-css",
            start: 20,
            end: 60,
        }];

        assert_eq!(simports, expected_simports);
    }

    #[test]
    fn content_between_tags() {
        let tag_import_chapter = make_tag_import_chapter();

        let simport = &SuperImport::parse_chapter(&tag_import_chapter)[0];

        let chapter_dir = "book/src/test-cases/tag-import";
        let chapter_dir = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), chapter_dir);

        let content_between_tags = simport.read_content_between_tags(&chapter_dir.into());

        let expected_content = r#".this-will-be-included {
  display: block;
}"#;

        assert_eq!(content_between_tags.unwrap(), expected_content);
    }

    #[test]
    fn replace_chapter() {
        let mut tag_import_chapter = make_tag_import_chapter();
        let mut item = BookItem::Chapter(tag_import_chapter);

        process_chapter(&mut item, &"".into());

        let expected_content = r#"# Tag Import

```md
.this-will-be-included {
  display: block;
}
```
"#;
        match item {
            BookItem::Chapter(tag_import_chapter) => {
                assert_eq!(tag_import_chapter.content.as_str(), expected_content);
            }
            _ => panic!(""),
        };
    }

    #[test]
    fn replace_escaped_simport() {
        let mut escaped_import_chapter = make_escaped_import_chapter();

        let expected_content = escaped_import_chapter.content.clone();
        let expected_content = r#"# Escaped Sinclude

```
\{{#sinclude ./ignored.txt@foo-bar}}
```
"#;

        let mut item = BookItem::Chapter(escaped_import_chapter);

        process_chapter(&mut item, &"".into());

        match item {
            BookItem::Chapter(escaped_chapter) => {
                assert_eq!(escaped_chapter.content.as_str(), expected_content);
            }
            _ => panic!(""),
        };
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

    // Create a chapter to represent our Escaped test case in the /book
    // directory in this repo.
    fn make_escaped_import_chapter() -> Chapter {
        let chapter = "book/src/test-cases/escaped/README.md";

        let tag_import_chapter = Chapter::new(
            "Escaped",
            include_str!("../book/src/test-cases/escaped/README.md").to_string(),
            &format!("{}/{}", env!("CARGO_MANIFEST_DIR"), chapter),
            vec![],
        );

        tag_import_chapter
    }
}

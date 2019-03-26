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

    // Iterate backwards through the simports so that we start by replacing the imports
    // that are lower in the file first.
    //
    // This ensures that as we replace simports we aren't throwing off the start and end
    // indices of other simports.
    for simport in simports.iter().rev() {
        let new_content = match simport.read_content_between_tags() {
            Ok(new_content) => new_content,
            Err(err) => panic!("{:#?}", err), // FIXME: Return failure with `?`
        };

        content = content.replace(simport.full_simport_text, &new_content);
    }

    chapter.content = content;

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
    /// The text of this simport in the host_chapter
    ///
    /// {{ #simport some-file.txt@some-tag }}
    full_simport_text: &'a str,
    /// Tags after the characters after an `@` symbol. When importing from a file
    /// Superimport will pull all text before and after the `@tag`
    ///
    /// Some(super-section)
    tag: &'a str,
    /// Where in the chapter's bytes does this simport start?
    start: usize,
    /// Where in the chapter's bytes does this simport end?
    end: usize,
}

// Wrapping in lazy_static ensures that our regex is only compiled once
lazy_static! {
  /// The regex that finds simports such as -> `{{ #simport some-file.txt@some-tag }}`
  static ref RE: Regex = Regex::new(
  r"(?x)                        # (?x) means insignificant whitespace mode
                                # allows us to put comments and space things out.

    \\\{\{\#.*\}\}                # escaped import such as `\{{ #simport some-file.txt@some-tag }}`

  |                               # OR

                                  # Non escaped import -> `{{ #simport some-file.txt@some-tag }}`
    \{\{\s*                         # opening braces and whitespace
    \#simport                       # #simport
    \s+                             # separating whitespace
    (?P<file>[a-zA-Z0-9\s_.\-/\\]+) # some-file.txt
    @                               # @ symbol that denotes the name of a tag
    (?P<tag>[a-zA-Z0-9_.\-]+)       # some-tag (alphanumeric underscores and dashes)
    \s*\}\}                         # whitespace and closing braces
  "
  ).unwrap();
}

impl<'a> Simport<'a> {
    fn parse_chapter<'c>(chapter: &'c Chapter) -> Vec<Simport<'c>> {
        let mut simports = vec![];

        let matches = RE.captures_iter(chapter.content.as_str());

        for capture_match in matches {
            // {{#simport ./fixture.css@cool-css }}
            let full_capture = capture_match.get(0).unwrap();
            let file = capture_match["file"].into();
            let tag = capture_match.get(2).unwrap();

            let simport = Simport {
                host_chapter: chapter,
                file,
                full_simport_text: &chapter.content[full_capture.start()..full_capture.end()],
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
    #[fail(display = "Could not find `@simport start {}`", tag)]
    MissingStartTag { tag: String },
}

impl<'a> Simport<'a> {
    // TODO: Clean up - don't need 3 iterations through the file.. Do it in for loop.
    fn read_content_between_tags(&self) -> Result<String, TagError> {
        let tag = self.tag;

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
            full_simport_text: "{{#simport ./fixture.css@cool-css }}",
            tag: "cool-css",
            start: 20,
            end: 56,
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

    #[test]
    fn replace_chapter() {
        let mut tag_import_chapter = make_tag_import_chapter();

        process_chapter(&mut tag_import_chapter);

        let expected_content = r#"# Tag Import

```md
.this-will-be-included {
  display: block;
}
```
"#;
        assert_eq!(tag_import_chapter.content.as_str(), expected_content);
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

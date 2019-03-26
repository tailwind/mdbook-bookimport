# Introduction

When working on a book/guide for a repository you'll sometimes find yourself wanting to
import some of your source code into your guide so that you can discuss it or provide
higher level context.

`mdbook` comes with a way to `#include` in between two line numbers in a file - but this
can be prone to link rot when you modify a file but forget to modify all of the sections
in your guide that refer to a file.

 `Superimport` seeks to address this by allowing you to import sections of a file by annotating
 the file.

 This allows you to modify the code between the annotations as much as you like and the import will
 still behave as you originally intended.

`Superimport` was originally created to close [mdbook issue #879](https://github.com/rust-lang-nursery/mdBook/issues/879).

## Installation

```sh
cargo install mdbook-superimport
```

## In your book.toml

```md
{{#superimport ../book.toml@super-section }}
```

## Usage

Annotate any file with.

```rust
// @superimport start some-tag
// ... contents go here ...
// @superimport end some-tag
```

Superimport only looks for the `@superimport {start,end} some-tag`, so depending on
the file type that you're in you'll want to comment thoe annotation out
appropriately.

---

Here's how to use superimport to import a section of a file
labeled `super-section`.

```sh
# Without the back slash
\{{#superimport ../book.toml@super-section }}
```

```css

# @superimport start super-section
src = "src"
title = "The Mdbook Superimport Book"
# @superimport end super-section
```

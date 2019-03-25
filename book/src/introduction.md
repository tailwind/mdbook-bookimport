# Introduction

 When working on a book/guide for a repository you'll sometimes find yourself wanting to
 import some of your source code into your guide so that you can discuss it or provide
 higher level context.

`mdbook` comes with a way to `#include` in between two line numbers in a file - but this
can be prone to link rot when you modify a file but forget to modify all of the sections
in your guide that refer to a file.

 `Superimport` seeks to address this by allowing you to import sections of a file based on
words in the file.

`Superimport` was originally created to close
[mdbook issue #879](https://github.com/rust-lang-nursery/mdBook/issues/879).

## Installation

## In your book.toml

```md
{{#simport ../book.toml@super-section }}
```

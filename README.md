mdbook-bookimport
=====

[![Build status](https://circleci.com/gh/tailwind/mdbook-bookimport.svg?style=shield&circle-token=:circle-token)](https://circleci.com/gh/tailwind/mdbook-bookimport)

> Import code/text from other files into your mdbook - without the link rot.

## Background / Initial Motivation

`mdbook-bookimport` started as an issue in [mdbook #879](https://github.com/rust-lang-nursery/mdBook/issues/879).

At this time the default `#include` preprocessor in `mdbook` only supports importing smaller sections of a file by specifying
line numbers - so if you're including pieces of files that are actively maintained/changed you end up forgetting to update
the line numbers of your imports as your files change.

`mdbook-bookimport` allows you to import pieces of files based on text in the file - so that are you modify the file you continue
to import the code that you expected to.

## Quickstart

```sh
cargo install mdbook-bookimport
```

```toml
# In your book.toml
[preprocessor.bookimport]
```

```md
<!-- Your markdown file before processing -->

{{#bookimport ../path/to/file.foo@some-tag-name-here}}
```

```rust
// Some file named "file.foo"
fn main () {
  let not_imported = "This will NOT be imported!";

  // @book start some-tag-name-here

  // ...
  let imported = "This will be imported!"
  let also_imported = "Everyting between start/end gets imported."
  // ...

  // @book end some-tag-name-here
}
```

```md
<!-- Your markdown file after processing -->


  // ...
  let imported = "This will be imported!"
  let also_imported = "Everyting between start/end gets imported."
  // ...


```

## [Full Guide](https://tailwind.github.io/mdbook-bookimport/)

[The mdbook-bookimport guide](https://tailwind.github.io/mdbook-bookimport/)

## [API Documentation](https://tailwind.github.io/mdbook-bookimport/api/mdbook_bookimport)

[API](https://tailwind.github.io/mdbook-bookimport/api/mdbook_bookimport)

## Troubleshooting

If for some reason something ever went wrong for any reason..:

`RUST_LOG=debug mdbook build` would give more information.

## To Test

```sh
./test.sh
```

## See Also

- [mdbook](https://github.com/rust-lang-nursery/mdBook)

## License

Apache 2.0 / MIT

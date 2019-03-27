mdbook-superimport
=====

[![Build status](https://circleci.com/gh/tailwind/mdbook-superimport.svg?style=shield&circle-token=:circle-token)](https://circleci.com/gh/tailwind/mdbook-superimport)

> Import code/text from other files into your mdbook - without the link rot.

## Background / Initial Motivation

`mdbook-superimport` started as an issue in [mdbook #879](https://github.com/rust-lang-nursery/mdBook/issues/879).

At this time the default `#include` preprocessor in `mdbook` only supports importing smaller sections of a file by specifying
line numbers - so if you're including pieces of files that are actively maintained/changed you end up forgetting to update
the line numbers of your imports as your files change.

`mdbook-superimport` allows you to import pieces of files based on text in the file - so that are you modify the file you continue
to import the code that you expected to.

## Quickstart

```sh
cargo install mdbook-superimport
```

```toml
# In your book.toml
[preprocessor.superimport]
```

```md
<!-- In your markdown files -->

\`\`\`
{{#superimport ../path/to/file.foo@some-tag-name-here}}
\`\`\`
```

```rust
// Some file named "file.foo"
fn main () {
  let not_imported = "This will NOT be imported!";

  // @superimport start some-tag-name-here

  // ...
  let imported = "This will be imported!"
  let also_imported = "Everyting between start/end gets imported."
  // ...

  // @superimport end some-tag-name-here
}
```

## [Full Guide](https://tailwind.github.io/mdbook-superimport/)

[The mdbook-superimport guide](https://tailwind.github.io/mdbook-superimport/)

## [API Documentation](https://tailwind.github.io/mdbook-superimport/api/mdbook_superimport)

[API](https://tailwind.github.io/mdbook-superimport/api/mdbook_superimport)

## To Test

```sh
cargo test --all
```

## See Also

- [mdbook](https://github.com/rust-lang-nursery/mdBook)

## License

Apache 2.0 / MIT

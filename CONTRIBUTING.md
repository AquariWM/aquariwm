<!-- This Source Code Form is subject to the terms of the Mozilla Public
   - License, v. 2.0. If a copy of the MPL was not distributed with this
   - file, You can obtain one at https://mozilla.org/MPL/2.0/. -->

This contributor's document is not complete, and will be fleshed out further to include
information about the structure of AquariWM itself, how it works, the goals it sets out
to achieve and the plans that have been created for its features and code that have not
yet been implemented.

# License headers
AquariWM's core is licensed under the MPL-2.0 license, and requires that license headers
be present at the beginning of all source code added by contributors to the project.

AquariWM's core is licensed under the Mozilla Public License v2.0 (MPL-2.0) license.
As stated by Mozilla:

> This license must be used for all new code, unless the containing project, module or
externally-imported codebase uses a different license. If you can't put a header in the
file due to its structure, please put it in a LICENSE file in the same directory.

all new code contributed to AquariWM's core is required to include the following license
headers at the beginning of all relevant files. If the file format in question does not
support comments, please include a `LICENSE` file in the same directory with the
plain-text version of this header.

### Rust, and others with C-like comments
```rust
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
```

### Shell scripts, and others
```bash
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
```

### Markdown, HTML, and other markup languages
```markdown
<!-- This Source Code Form is subject to the terms of the Mozilla Public
   - License, v. 2.0. If a copy of the MPL was not distributed with this
   - file, You can obtain one at https://mozilla.org/MPL/2.0/. -->
```

### `LICENSE` files, and others
```
This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at https://mozilla.org/MPL/2.0/.
```

# Code style
It is important to maintain a consistent code style through the AquariWM project, and
among various code editors. As such, please stick to the following style guidelines
wherever possible:

## Whitespace and indentation
 - **Lines of code have a strict maximum length of 99 characters,** except where absolutely
   necessary. Your editor may automatically configure this option when opening the
   AquariWM project.
 - **Use 4 spaces for indentation, not tab characters.** We understand that there are
   accessibility issues that may be encountered with the use of spaces instead of tabs,
   but we must unfortunately stick with spaces so as to follow more general guidelines
   for the Rust language. You may configure your own tools that let you view 4-space
   indentation differently, or that can convert this indentation to tab characters while
   editing, and then back to 4 spaces when submitting.
 - **No trailing whitespace at the end of lines** (if a line only contains `x`, then it should
   be, in total, just `x` (rather than `x `, for example)
 - **No trailing whitespace should be used at the end of files**, including blank lines.

More information regarding whitespace can be found in
[the Rust code style guidelines](https://doc.rust-lang.org/1.0.0/style/style/whitespace.html).

## Curly brackets/braces
Opening brackets should always go on the same line as the block which they are opening. For
example (in Rust):
```rust
fn foo() {
   // ...
}
```
rather than
```rust
fn foo()
{
   // ...
}
```

More information and similar rules can be found in
[the Rust code style guidelines](https://doc.rust-lang.org/1.0.0/style/style/braces.html).

## Comments
### Line comments
Throughout the AquariWM project, **line comments are preferred to block comments**. Rather
than writing:

```rust
/*
 * Window managers are one of the core components of the modern Linux/BSD desktop. It is
 * not an exaggeration to say that they define to a large degree our day-to-day user
 * experience, as they are responsible for deciding how individual windows look, move around,
 * react to input, and organize themselves.
 */
```

you should instead write:

```rust
// Window managers are one of the core components of the modern Linux/BSD desktop. It is
// not an exaggeration to say that they define to a large degree our day-to-day user
// experience, as they are responsible for deciding how individual windows look, move around,
// react to input, and organize themselves.
```

### Doc comments
Full, complete, **high-quality documentation is considered extremely important** in the AquariWM
project, and is **required for all contributions which add or change any APIs or features**. Doc
comments can be written in Rust to easily document Rust code in a very straightforward manner.
For example:

```rust
/// This is a wonderful doc comment that plainly explains the following function to all readers.
/// It should describe the functionality of the accompanying code, and should clearly explain
/// how to use it. It should be easily understood by those who may not have experience in the
/// area it is commenting on, so do make sure to explain any terms or jargon _(area- or
/// field-specific terms)_.
///
/// Also, the first line of doc comments is used as a summary. Please do make that one quick and
/// to-the-point about what the documented code *is* and how it might relate to the reader.
fn my_wonderful_function_stub() {
   print!("Whoops! Forgot to write any actually good code!");
}
```

**If code is not necessarily public-facing, it should still be clearly commented** so that anyone
who may need or want to read it can understand what it does, why, and how to use it. Again, make
sure to consider that the reader may not understand the terms or concepts you use in your comment.
However, **please don't state the obvious** - you wouldn't do the following, for example:

```rust
// opens a file
fn open_file() {
   // ...
}
```

### More information
More information regarding comments can be found in
[the Rust code style guidelines](https://doc.rust-lang.org/1.0.0/style/style/comments.html).

## Naming conventions
**Naming items of code should follow the guidelines of its respective language.**

### Rust
Extensive information about naming conventions in Rust can be found in
[the Rust code style guidelines](https://doc.rust-lang.org/1.0.0/style/style/naming/README.html),
and the what follows is some paraphrasings and extracts from that page.

#### General conventions
**In Rust, `CamelCase` shall be used for "type-level" constructs (types and traits), and `snake_case`
for "value-level" constructs.** More precisely, the following table is provided by Rust:

|Item                   |Convention                                                 |
|-----------------------|-----------------------------------------------------------|
|Crates                 |`snake_case` (but prefer single word)                      |
|Modules                |`snake_case`                                               |
|Types                  |`CamelCase`                                                |
|Traits                 |`CamelCase`                                                |
|Enum variants          |`CamelCase`                                                |
|Functions              |`snake_case`                                               |
|Methods                |`snake_case`                                               |
|General constructors   |`new` or `with_more_details`                               |
|Conversion constructors|`from_some_other_type`                                     |
|Local variables        |`snake_case`                                               |
|Static variables       |`SCREAMING_SNAKE_CASE`                                     |
|Constant variables     |`SCREAMING_SNAKE_CASE`                                     |
|Type parameters        |concise `CamelCase`, usually single uppercase letter: `T`  |
|Lifetimes              |short, lowercase: `'a`                                     |

When using `CamelCase`, acronyms count as one word and shall be capitalized as such: use `Uuid`
instead of `UUID`. In `snake_case`, acronyms are lower-cased: `is_xid_start`.

In `snake_case` and `SCREAMING_SNAKE_CASE`, a word should never consist of only a single
letter, unless it is the last word in the name. For example, `btree_map` is used rather than
`b_tree_map`, but `PI_2` is used rather than `PI2`.

#### Avoid redundant prefixes
Names of items within a module (as in, the concept of a module in Rust with `mod`) should not be
prefixed with that module's name. Prefer:

```rust
mod foo {
   pub struct Error { ... }
}
```
over
```rust
mod foo {
   pub struct FooError { ... }
}
```

This is primarily to avoid repetitive references to module items (known as stuttering), such as
`foo::FooError`.

<!-- This Source Code Form is subject to the terms of the Mozilla Public
   - License, v. 2.0. If a copy of the MPL was not distributed with this
   - file, You can obtain one at https://mozilla.org/MPL/2.0/. -->

# Please note...
This document is only temporary, and should be replaced by a contributors' wiki. To summarise code
style requirements (until the wiki contains such information):
 - AquariWM uses _hard tab characters_ throughout the project for indentation, rather than spaces.
   More information about this can be found in the pinned discussion in the Discussions tab of the
   GitHub repository (AquariWM/aquariwm). `EditorConfig` and `rustfmt` are configured to support
   this already, and we recommend that you install a plugin for your editor to recognise
   `.editorconfig` files automatically, as well as to format your Rust code automatically with
   `cargo fmt`.
 - The code style guidelines of the language you are using should be used _unless_ we specify
   otherwise; for instance, as we specify that hard tab characters shall be used, that takes
   preference over Rust's official code style guidelines.

You can find code style guidelines for Rust
[here](https://doc.rust-lang.org/1.0.0/style/README.html). For license header information, please
read ahead.

## License headers
AquariWM's core is licensed under the Mozilla Public License v2.0 (MPL-2.0) license. As stated by
Mozilla:

> This license must be used for all new code, unless the containing project, module or
externally-imported codebase uses a different license. If you can't put a header in the file due to
its structure, please put it in a LICENSE file in the same directory.

All new code contributed to AquariWM's core is required to include the following license headers at
the beginning of all relevant files. If the file format in question does not support comments,
please include a `LICENSE` file in the same directory with the plain-text version of this header.

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
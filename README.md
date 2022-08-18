<!-- This Source Code Form is subject to the terms of the Mozilla Public
   - License, v. 2.0. If a copy of the MPL was not distributed with this
   - file, You can obtain one at https://mozilla.org/MPL/2.0/. -->

AquariWM is a window manager focused on complete modularity. It aims to define and implement a
number of X protocol specifions for various core window manager features, allowing different module
clients to work together. For example, AquariWM defines a 'decorator' specifcation that allows window
decorations to be defined as separate clients, independently of any compliant window manager. That
means you can 'mix-and-match' the appearance of window decorations completely independently of the
window manager.

AquariWM is currently in its very early stages. Further development on AquariWM is waiting on
[X.RS](https://github.com/XdotRS/xrs), an X library for Rust that is being made for AquariWM. A big
focus for both X.RS' code and AquariWM's, however, is thorough and extensive comments and
documentation. The goal is that you should be able to read the docs and/or code and be able to
understand what's happening, without requiring much prior knowledge of the internals of X or a window
manager.

AquariWM is created to be a community project: when there is more of an actual code foundation for the
window manager ready, the intention is for it to be largely developed through community contributions.

## When will AquariWM be ready for further development?
I suspect sometime around December 2022 might be when the X library can be used to further develop
AquariWM in a very much pre-alpha state. Stay tuned!

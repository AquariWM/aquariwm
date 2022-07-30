// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use xcb::x::Window;

/// TODO: Stub
struct Monitor {
    desktops: Vec<Desktop>,
}

/// TODO: Stub
struct Desktop {
    workspace: Workspace,
}

/// TODO: Stub
struct Workspace {
    windows: Vec<Window>,
}

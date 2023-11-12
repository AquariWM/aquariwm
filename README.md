<!-- This Source Code Form is subject to the terms of the Mozilla Public
   - License, v. 2.0. If a copy of the MPL was not distributed with this
   - file, You can obtain one at https://mozilla.org/MPL/2.0/. -->

# AquariWM

AquariWM will be an X11 window manager and Wayland compositor focused on a highly modular design. It will also focus on
high-quality documentation and being accessible and welcoming to all.

## Modularity

Tiling window layout managers will be able to be provided by external applications ('AquariWM clients'). AquariWM
clients will also be able to provide decoration managers, providing the window decorations for windows.

Other areas of modularity will exist too but ideas for them will become clear as AquariWM is developed - areas could
include input methods, config/settings managers, or adding modularity that X11 already has to the AquariWM Wayland
compositor (window managers and compositors being provided by external clients being one example).

## Current state

At the time of writing (November 2023), AquariWM development is in early stages, with the basics of a running X11 window
manager and Wayland compositor achieved. Work is focused on implementing the tiling layout manager design, starting with
simply using traits for layout managers, with the goal of transitioning to use a custom protocol in the future (the
specifics of which are yet to be decided).

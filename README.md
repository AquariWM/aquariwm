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

At the time of writing (December 2023), AquariWM development is in early stages, though the layout manager system is
implemented in Rust (with the goal of transitioning to use a custom protocol in the future, the specifics of which are
yet to be decided). @Antikyth, the only author of AquariWM at the time of writing, is working on
[`generational-arena-tree`], a tree implementation in Rust that gives the flexibility to implement more complex features
for tiling layouts (e.g. taking windows' minimum and maximum sizes into account). Specifically, it allows:
- nodes to be mutated directly,
- nodes to be iterated over mutably,
- nodes to be split by type into separate branches (nodes that may have children) and leaves (nodes that may not have
  children), which each have their own associated data type.
  - This is required because, in window layouts, every branch has an orientation, and every leaf has a window. No branch
    may have a window, and no leaf may have an orientation.

Here is a screenshot of a working Main + Stack layout manager implemented in the current state of AquariWM:
![A picture of a Main + Stack layout manager functioning in AquariWM, with window gaps enabled](https://cdn.discordapp.com/attachments/1012049086121246843/1176465058449076294/image.png?ex=657831f7&is=6565bcf7&hm=be348cc7313d69a9da3f1b5bb39dde9ef2261a679034438aa45eefc5d423b0c4&)

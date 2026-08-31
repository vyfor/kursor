# kursor

a retained-mode terminal ui library for rust.

## motivation

kursor emerged from a personal need to build tui applications where components would stay self-contained.

application state should not need to know about widget handles, and widgets should not need to push state through every intermediate parent. there should be a shared state model that both the ui and application logic can observe and mutate effortlessly.

kursor is an attempt to make that model practical without delegating much of that plumbing onto the user.

the project is still early and the api is subject to change.

## architecture

the project is split into several crates:
- `kursor-core`: a lower-level library that provides mechanisms and primitives.
- `kursor`: a higher-level framework that builds on top of `kursor-core` and decides how those mechanisms are presented and combined.
- `kursor-fx`: visual effects layer for kursor.
- `kursor-image`: terminal graphics protocol implementations.

## concepts

### state

- atoms: reactive state that can be shared between different parts of an application.
- signals: reactive state local to the ui.
- memo: derived state from a signal.
- animated: values that gradually interpolate over time.

### widgets

small, composable building blocks. kursor is built to be extended. it is grounded on the idea that if a feature can be a widget, it is a widget. themes, overlays, keybindings and such are all simple widgets abstracted over core.

### input handling

events are dispatched in two phases: capture (root to target) and bubble (target to root). kursor provides a primitive focus mechanism, with the focused component serving as the event target.

aside from handling events locally, widgets can opt into global input handling while remaining self-contained.

### behavior & intents

it is generally a bad idea for a widget to hard-code the interpretation of its input. instead, kursor exposes a *behavior* that translates raw input into a semantic *intent*. the component then decides how that intent alters its state.

### animations & effects

kursor tightly integrates with [`animate`](https://github.com/vyfor/animate) to both let *you* animate state without typical boilerplate, and let *widgets* to opt in to this system with very little wiring.

`kursor-fx`, on the other hand, provides primitives to construct and apply visual effects to any component. animations change the internal state of widgets, whilst effects operate externally on the rendered output.

### images

terminal graphics via kitty, sixel, iterm2 protocols, as well as good old halfblocks. and yes, kursor is able to do the terminal probing for you to detect its capabilities.

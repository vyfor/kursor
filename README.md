# kursor

a retained-mode terminal ui library for rust.

## motivation

kursor emerged from a personal need to build tui applications where components would stay self-contained.

application state should not need to know about widget handles, and widgets should not need to push state through every intermediate parent. there should be a shared state model that both the ui and application logic can observe and mutate effortlessly.

kursor is an attempt to make that model practical without delegating much of that plumbing onto the user.

the project is still early and the api is subject to change.

## architecture

the project is split into two crates:
- `kursor-core`: a lower-level library that provides mechanisms and primitives.
- `kursor`: a higher-level framework that builds on top of `kursor-core` and decides how those mechanisms are presented and combined.

## concepts

### atoms

reactive state that can be shared between different parts of an application.

### environment

a typemap for components to provide values to their descendants or retrieve values from their ancestors.

### widgets

small, composable building blocks. kursor is built on the idea that if a feature can be a widget, it is a widget. themes, overlays, keybindings and such are all simple widgets abstracted over core.

### input handling

events are dispatched in two phases: capture (root to target) and bubble (target to root). kursor provides a primitive focus mechanism, with the focused component serving as the event target.

aside from handling events locally, widgets can opt into global input handling while remaining self-contained.

### behavior & intents

in most cases, it is a bad idea for any widget to hard-code its event handling. instead, kursor exposes a **behavior** that translates raw input into a semantic **intent**. the component then decides how that intent alters its state.
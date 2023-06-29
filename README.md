# proctmux

A TUI utility for running multiple commands in parallel within tmux panes.

This app is intended to be a drop-in replacement of [procmux](https://github.com/napisani/procmux) for tmux users. Utilizing tmux panes/windows gives the user a more powerful and familiar environment for managing their long-running processes.

## Proof of Concept

Current version is just a proof of concept. There are still a few things left to test related to processes, such as graceful shutdowns, interrupts, and detecting finished processes. Once that is complete code cleanup and feature building will commence.

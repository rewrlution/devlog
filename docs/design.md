# Design

This doc captures the design decision.

## Storage

We use `event sourcing` pattern for the storage layer.

- `events`: this folder contains all append-only events files
- `entries`: this folder contains the markdown files, which can be overwritten

Check `src/storage.rs` for more details.

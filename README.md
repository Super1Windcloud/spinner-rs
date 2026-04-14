# spinner-rs

[中文说明](./README.zh-CN.md)

A lightweight Rust spinner library for terminal applications. It helps you show progress while work is running and persist a final success, failure, warning, or info state when the task finishes.

## Features

- Background-thread-driven terminal animation
- Update text, prefix, and suffix while running
- Built-in `succeed`, `fail`, `warn`, and `info` states
- Custom frame sequences and refresh intervals
- Color, indent, cursor visibility, and output stream controls
- Automatic TTY detection

## Installation

```toml
[dependencies]
ora-rs = { path = "." }
```

Until the crate is published, use it as a local path dependency.

## Quick Start

```rust
use std::{io, thread, time::Duration};

use ora_rs::Spinner;

fn main() -> io::Result<()> {
    let mut spinner = Spinner::new("Connecting to service");

    spinner.start()?;
    thread::sleep(Duration::from_secs(1));

    spinner.set_text("Downloading data");
    thread::sleep(Duration::from_secs(1));

    spinner.succeed(Some("Completed"))?;
    Ok(())
}
```

## Builder API

```rust
use std::{io, thread, time::Duration};

use ora_rs::{Color, Spinner, SpinnerStyle, Stream};

fn main() -> io::Result<()> {
    let mut spinner = Spinner::builder()
        .text("Building assets")
        .prefix_text("worker-1")
        .suffix_text("queued")
        .color(Color::Yellow)
        .spinner(SpinnerStyle::arrow())
        .indent(2)
        .hide_cursor(true)
        .stream(Stream::Stderr)
        .build();

    spinner.start()?;
    thread::sleep(Duration::from_secs(2));
    spinner.succeed(Some("Assets ready"))?;
    Ok(())
}
```

If you want to compose configuration first and create the spinner later, use `SpinnerBuilder::build_options()` or `SpinnerOptions::builder()`.

## Examples

```bash
cargo run --example basic
cargo run --example custom_style
```

## Testing

```bash
cargo test
```

## API Overview

- `Spinner::new(text)`
- `Spinner::builder()`
- `Spinner::with_options(options)`
- `SpinnerBuilder::build()`
- `SpinnerBuilder::build_options()`
- `start()`
- `stop()`
- `clear()`
- `snapshot()`
- `set_text(...)`
- `set_prefix_text(...)`
- `set_suffix_text(...)`
- `set_color(...)`
- `set_spinner(...)`
- `succeed(...)`
- `fail(...)`
- `warn(...)`
- `info(...)`
- `stop_and_persist(...)`

use std::{io, thread, time::Duration};

use ora_rs::{Color, Spinner, SpinnerStyle, Stream};

fn main() -> io::Result<()> {
    let mut spinner = Spinner::builder()
        .text("Bootstrapping spinner-rs")
        .prefix_text("demo")
        .suffix_text("starting")
        .color(Color::Cyan)
        .spinner(SpinnerStyle::dots())
        .indent(2)
        .hide_cursor(true)
        .stream(Stream::Stderr)
        .build();

    spinner.start()?;

    thread::sleep(Duration::from_millis(900));
    spinner.set_text("Loading configuration");

    thread::sleep(Duration::from_millis(900));
    spinner.set_suffix_text("warming up workers");

    thread::sleep(Duration::from_millis(900));
    spinner.set_spinner(SpinnerStyle::arrow());
    spinner.set_color(Some(Color::Yellow));
    spinner.set_text("Processing tasks");

    thread::sleep(Duration::from_millis(1200));
    spinner.succeed(Some("All tasks completed"))?;

    Ok(())
}

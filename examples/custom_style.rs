use std::{io, thread, time::Duration};

use ora_rs::{Color, Spinner, SpinnerStyle, Stream};

fn main() -> io::Result<()> {
    let mut spinner = Spinner::builder()
        .text("Processing build graph")
        .prefix_text("pipeline")
        .suffix_text("phase-1")
        .color(Color::Yellow)
        .spinner(SpinnerStyle::arrow())
        .indent(4)
        .stream(Stream::Stderr)
        .build();

    spinner.start()?;
    thread::sleep(Duration::from_millis(900));

    spinner.set_suffix_text("phase-2");
    spinner.set_text("Packing release artifacts");
    thread::sleep(Duration::from_millis(900));

    spinner.set_color(Some(Color::Green));
    spinner.succeed(Some("Release artifacts ready"))?;
    Ok(())
}

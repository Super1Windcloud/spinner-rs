use std::{thread, time::Duration};

use spinner_rs::{Color, Spinner, SpinnerBuilder, SpinnerStyle, Stream};

#[test]
fn builder_creates_spinner_with_expected_snapshot() {
    let spinner = Spinner::builder()
        .text("indexing")
        .prefix_text("job-1")
        .suffix_text("queued")
        .color(Color::Yellow)
        .spinner(SpinnerStyle::line())
        .indent(3)
        .hide_cursor(false)
        .enabled(false)
        .stream(Stream::Stdout)
        .build();

    let snapshot = spinner.snapshot();
    assert_eq!(snapshot.text, "indexing");
    assert_eq!(snapshot.prefix_text, "job-1");
    assert_eq!(snapshot.suffix_text, "queued");
    assert_eq!(snapshot.color, Some(Color::Yellow));
    assert_eq!(snapshot.spinner, SpinnerStyle::line());
    assert_eq!(snapshot.indent, 3);
    assert!(!snapshot.hide_cursor);
    assert!(!snapshot.enabled);
    assert_eq!(snapshot.stream, Stream::Stdout);
    assert!(!snapshot.is_spinning);
}

#[test]
fn builder_supports_no_color_and_runtime_lifecycle() {
    let mut spinner = SpinnerBuilder::new()
        .text("compile")
        .no_color()
        .enabled(false)
        .build();

    spinner.start().expect("start should succeed");
    assert!(spinner.is_spinning());

    thread::sleep(Duration::from_millis(10));
    spinner
        .set_text("package")
        .set_prefix_text("worker")
        .set_suffix_text("done");

    let snapshot = spinner.snapshot();
    assert_eq!(snapshot.text, "package");
    assert_eq!(snapshot.prefix_text, "worker");
    assert_eq!(snapshot.suffix_text, "done");
    assert_eq!(snapshot.color, None);
    assert!(snapshot.is_spinning);

    spinner
        .succeed(Some("complete"))
        .expect("succeed should stop");
    assert!(!spinner.is_spinning());
    assert_eq!(spinner.text(), "complete");
}

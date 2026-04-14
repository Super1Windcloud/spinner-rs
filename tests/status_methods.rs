use ora_rs::{Color, Spinner};

#[test]
fn status_methods_replace_text_and_stop_spinner() {
    let mut spinner = Spinner::builder().text("boot").enabled(false).build();

    spinner.start().expect("start should succeed");
    spinner.fail(Some("failed")).expect("fail should succeed");
    assert_eq!(spinner.text(), "failed");
    assert!(!spinner.is_spinning());

    spinner.start().expect("restart should succeed");
    spinner.warn(Some("warning")).expect("warn should succeed");
    assert_eq!(spinner.text(), "warning");
    assert!(!spinner.is_spinning());
}

#[test]
fn snapshot_reports_color_changes() {
    let mut spinner = Spinner::new("sync");
    spinner.set_color(Some(Color::Blue));

    let snapshot = spinner.snapshot();
    assert_eq!(snapshot.color, Some(Color::Blue));
    assert_eq!(snapshot.text, "sync");
}

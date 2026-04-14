use std::{io, thread, time::Duration};

use ora_rs::Spinner;

fn main() -> io::Result<()> {
    let mut spinner = Spinner::new("Preparing workspace");

    spinner.start()?;
    thread::sleep(Duration::from_millis(800));

    spinner.set_text("Fetching dependencies");
    thread::sleep(Duration::from_millis(800));

    spinner.set_text("Running checks");
    thread::sleep(Duration::from_millis(800));

    spinner.succeed(Some("Workspace ready"))?;
    Ok(())
}

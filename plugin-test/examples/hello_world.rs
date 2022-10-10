use std::{time::Duration, thread};
use plugin_test::{App};

fn main() -> anyhow::Result<()> {
    let mut app = App::new();
    println!("Successfully created App");

    app.load_handler()?;
    println!("Successfully loaded handler");

    for _i in 0..5 {
        app.do_something()?;
        thread::sleep(Duration::from_secs(1));
        println!("did something");
    }

    app.unload_handler()?;
    println!("Successfully unloaded handler");

    Ok(())
}
mod app;

use anyhow::{Error, Result};
use app::App;

fn main() -> Result<(), Error> {
    let mut app = App::new();

    smol::block_on(async {
        _ = app.run().await;
    });

    Ok(())
}

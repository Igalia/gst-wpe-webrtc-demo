#![recursion_limit = "256"]

#[macro_use]
mod macros;
mod app;
mod janus;
mod pipeline;
mod settings;
mod utils;

use crate::app::App;
#[macro_use]
extern crate log;

pub const APPLICATION_NAME: &str = "com.igalia.gstwpe.broadcast.demo";

async fn async_main() -> Result<(), anyhow::Error> {
    gst::init()?;
    let app = App::new()?;
    app.run().await
}

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    let main_context = glib::MainContext::default();
    main_context.block_on(async_main())
}

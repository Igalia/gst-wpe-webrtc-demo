use crate::janus;
use crate::pipeline::Pipeline;

use anyhow::anyhow;
use gst::prelude::*;
use std::ops;
use std::rc::Rc;

// Our refcounted application struct for containing all the state we have to carry around.
//
// This represents our main application window.
#[derive(Clone)]
pub struct App(Rc<AppInner>);

// Deref into the contained struct to make usage a bit more ergonomic
impl ops::Deref for App {
    type Target = AppInner;

    fn deref(&self) -> &AppInner {
        &*self.0
    }
}

pub struct AppInner {
    pipeline: Pipeline,
}

impl App {
    pub fn new() -> Result<Self, anyhow::Error> {
        let pipeline =
            Pipeline::new().map_err(|err| anyhow!("Error creating pipeline: {:?}", err))?;

        let app = App(Rc::new(AppInner { pipeline }));
        Ok(app)
    }

    pub async fn run(&self) -> Result<(), anyhow::Error> {
        self.pipeline.prepare()?;
        let bin = self.pipeline.pipeline.clone().upcast::<gst::Bin>();
        let mut gw = janus::JanusGateway::new(bin).await?;
        self.pipeline.start()?;
        gw.run().await?;
        Ok(())
    }
}

// Make sure to shut down the pipeline when it goes out of scope
// to release any system resources
impl Drop for AppInner {
    fn drop(&mut self) {
        let _ = self.pipeline.stop();
    }
}

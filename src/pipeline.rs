use crate::settings::VideoResolution;
use crate::utils;
use gst::{self, prelude::*};
use std::error;
use std::ops;
use std::rc::{Rc, Weak};

// Our refcounted pipeline struct for containing all the media state we have to carry around.
#[derive(Clone)]
pub struct Pipeline(Rc<PipelineInner>);

// Deref into the contained struct to make usage a bit more ergonomic
impl ops::Deref for Pipeline {
    type Target = PipelineInner;

    fn deref(&self) -> &PipelineInner {
        &*self.0
    }
}

pub struct PipelineInner {
    pub pipeline: gst::Pipeline,
}

// Weak reference to our pipeline struct
//
// Weak references are important to prevent reference cycles. Reference cycles are cases where
// struct A references directly or indirectly struct B, and struct B references struct A again
// while both are using reference counting.
pub struct PipelineWeak(Weak<PipelineInner>);
impl PipelineWeak {
    pub fn upgrade(&self) -> Option<Pipeline> {
        self.0.upgrade().map(Pipeline)
    }
}

impl Pipeline {
    pub fn new() -> Result<Self, Box<dyn error::Error>> {
        let settings = utils::load_settings();

        let (width, height) = match settings.video_resolution {
            VideoResolution::V480P => (640, 480),
            VideoResolution::V720P => (1280, 720),
            VideoResolution::V1080P => (1920, 1080),
        };

        let pipeline = gst::parse_launch(&format!(
            "webrtcbin name=webrtcbin stun-server=stun://stun2.l.google.com:19302 \
             glvideomixerelement name=mixer sink_1::zorder=0 sink_1::height={height} sink_1::width={width} \
             ! tee name=video-tee ! queue ! gtkglsink enable-last-sample=0 name=sink qos=0 \
             wpesrc location=http://127.0.0.1:3000 name=wpesrc draw-background=0 \
             ! capsfilter name=wpecaps caps=\"video/x-raw(memory:GLMemory),width={width},height={height},pixel-aspect-ratio=(fraction)1/1\" ! glcolorconvert ! queue ! mixer. \
             v4l2src name=videosrc ! capsfilter name=camcaps caps=\"image/jpeg,width={width},height={height},framerate=30/1\" !  queue ! jpegparse ! queue ! jpegdec ! videoconvert ! queue ! glupload ! glcolorconvert
             ! queue ! mixer. \
             ", width=width, height=height)
        )?;

        // Upcast to a gst::Pipeline as the above function could've also returned an arbitrary
        // gst::Element if a different string was passed
        let pipeline = pipeline
            .downcast::<gst::Pipeline>()
            .expect("Couldn't downcast pipeline");

        // Request that the pipeline forwards us all messages, even those that it would otherwise
        // aggregate first
        pipeline.set_property_message_forward(true);

        let pipeline = Pipeline(Rc::new(PipelineInner { pipeline }));

        // Install a message handler on the pipeline's bus to catch errors
        let bus = pipeline.pipeline.get_bus().expect("Pipeline had no bus");

        // GStreamer is thread-safe and it is possible to attach bus watches from any thread, which
        // are then nonetheless called from the main thread. So by default, add_watch() requires
        // the passed closure to be Send. We want to pass non-Send values into the closure though.
        //
        // As we are on the main thread and the closure will be called on the main thread, this
        // is actually perfectly fine and safe to do and we can use add_watch_local().
        // add_watch_local() would panic if we were not calling it from the main thread.
        let pipeline_weak = pipeline.downgrade();
        bus.add_watch_local(move |_bus, msg| {
            let pipeline = upgrade_weak!(pipeline_weak, glib::Continue(false));

            pipeline.on_pipeline_message(msg);

            glib::Continue(true)
        })
        .expect("Unable to add bus watch");

        Ok(pipeline)
    }

    // Downgrade to a weak reference
    pub fn downgrade(&self) -> PipelineWeak {
        PipelineWeak(Rc::downgrade(&self.0))
    }

    pub fn prepare(&self) -> Result<gst::StateChangeSuccess, gst::StateChangeError> {
        let settings = utils::load_settings();
        let webrtc_codec = settings.webrtc_codec_params();
        let bin_description = &format!(
            "queue name=webrtc-vqueue ! gldownload ! videoconvert ! {encoder} ! {payloader} ! queue ! capsfilter name=webrtc-vsink caps=\"application/x-rtp,media=video,encoding-name={encoding_name},payload=96\"",
            encoder=webrtc_codec.encoder, payloader=webrtc_codec.payloader,
            encoding_name=webrtc_codec.encoding_name
        );

        let bin = gst::parse_bin_from_description(bin_description, false).unwrap();
        bin.set_name("webrtc-vbin").unwrap();

        let video_queue = bin
            .get_by_name("webrtc-vqueue")
            .expect("No webrtc-vqueue found");
        let video_tee = self
            .pipeline
            .get_by_name("video-tee")
            .expect("No video-tee found");

        self.pipeline
            .add(&bin)
            .expect("Failed to add recording bin");

        let srcpad = video_tee
            .get_request_pad("src_%u")
            .expect("Failed to request new pad from tee");
        let sinkpad = video_queue
            .get_static_pad("sink")
            .expect("Failed to get sink pad from recording bin");

        if let Ok(video_ghost_pad) = gst::GhostPad::new(Some("video_sink"), &sinkpad) {
            bin.add_pad(&video_ghost_pad).unwrap();
            srcpad.link(&video_ghost_pad).unwrap();
        }

        let webrtcbin = self.pipeline.get_by_name("webrtcbin").unwrap();
        let sinkpad2 = webrtcbin.get_request_pad("sink_%u").unwrap();
        let vsink = bin
            .get_by_name("webrtc-vsink")
            .expect("No webrtc-vqueue found");
        let srcpad = vsink.get_static_pad("src").unwrap();
        if let Ok(webrtc_ghost_pad) = gst::GhostPad::new(Some("webrtc_video_src"), &srcpad) {
            bin.add_pad(&webrtc_ghost_pad).unwrap();
            webrtc_ghost_pad.link(&sinkpad2).unwrap();
        }

        self.pipeline.set_state(gst::State::Ready)
    }

    pub fn start(&self) -> Result<gst::StateChangeSuccess, gst::StateChangeError> {
        // This has no effect if called multiple times
        self.pipeline.set_state(gst::State::Playing)
    }

    pub fn stop(&self) -> Result<gst::StateChangeSuccess, gst::StateChangeError> {
        // This has no effect if called multiple times
        self.pipeline.set_state(gst::State::Null)
    }

    // Here we handle all message we get from the GStreamer pipeline. These are notifications sent
    // from GStreamer, including errors that happend at runtime.
    //
    // This is always called from the main application thread by construction.
    fn on_pipeline_message(&self, msg: &gst::MessageRef) {
        use gst::MessageView;

        // A message can contain various kinds of information but
        // here we are only interested in errors so far
        match msg.view() {
            MessageView::Error(err) => {
                panic!(
                    "Error from {:?}: {} ({:?})",
                    err.get_src().map(|s| s.get_path_string()),
                    err.get_error(),
                    err.get_debug()
                );
            }
            MessageView::Application(msg) => match msg.get_structure() {
                // Here we can send ourselves messages from any thread and show them to the user in
                // the UI in case something goes wrong
                Some(s) if s.get_name() == "warning" => {
                    let text = s
                        .get::<&str>("text")
                        .expect("Warning message without text")
                        .unwrap();
                    panic!("{}", text);
                }
                _ => (),
            },
            MessageView::StateChanged(state_changed) => {
                if let Some(element) = msg.get_src() {
                    if element == self.pipeline {
                        let bin_ref = element.downcast_ref::<gst::Bin>().unwrap();
                        let filename = format!(
                            "gst-wpe-broadcast-demo-{:#?}_to_{:#?}",
                            state_changed.get_old(),
                            state_changed.get_current()
                        );
                        bin_ref.debug_to_dot_file_with_ts(gst::DebugGraphDetails::all(), filename);
                    }
                }
            }
            MessageView::AsyncDone(_) => {
                if let Some(element) = msg.get_src() {
                    let bin_ref = element.downcast_ref::<gst::Bin>().unwrap();
                    bin_ref.debug_to_dot_file_with_ts(
                        gst::DebugGraphDetails::all(),
                        "gst-wpe-broadcast-demo-async-done",
                    );
                }
            }
            _ => (),
        };
    }
}

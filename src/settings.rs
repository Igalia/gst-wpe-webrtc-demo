use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub enum VideoResolution {
    V480P,
    V720P,
    V1080P,
}

impl Default for VideoResolution {
    fn default() -> Self {
        VideoResolution::V720P
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub enum WebRTCCodec {
    VP8,
    VP9,
    H264,
}

impl Default for WebRTCCodec {
    fn default() -> Self {
        WebRTCCodec::VP8
    }
}

#[derive(Debug)]
pub struct VideoParameter {
    pub encoder: &'static str,
    pub encoding_name: &'static str,
    pub payloader: &'static str,
}

const VP8_PARAM: VideoParameter = VideoParameter {
    encoder: "vp8enc target-bitrate=400000 threads=4 overshoot=25 undershoot=100 deadline=33000 keyframe-max-dist=1",
    encoding_name: "VP8",
    payloader: "rtpvp8pay picture-id-mode=2"
};

const VP9_PARAM: VideoParameter = VideoParameter {
    encoder: "vp9enc target-bitrate=128000 undershoot=100 deadline=33000 keyframe-max-dist=1",
    encoding_name: "VP9",
    payloader: "rtpvp9pay picture-id-mode=2",
};

const H264_PARAM: VideoParameter = VideoParameter {
    //encoder: "x264enc tune=zerolatency",
    encoder: "vaapih264enc",
    encoding_name: "H264",
    payloader: "rtph264pay",
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Settings {
    pub video_resolution: VideoResolution,
    pub webrtc_codec: WebRTCCodec,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            //h264_encoder: "video/x-raw,format=NV12 ! vaapih264enc bitrate=20000 keyframe-period=60 ! video/x-h264,profile=main".to_string(),
            video_resolution: VideoResolution::default(),
            webrtc_codec: WebRTCCodec::default(),
        }
    }
}

impl Settings {
    pub fn webrtc_codec_params(&self) -> VideoParameter {
        match self.webrtc_codec {
            WebRTCCodec::VP8 => VP8_PARAM,
            WebRTCCodec::VP9 => VP9_PARAM,
            WebRTCCodec::H264 => H264_PARAM,
        }
    }
}

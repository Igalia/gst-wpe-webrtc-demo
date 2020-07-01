# GStreamer WPE Web Overlay WebRTC Broadcast demo

This application allows the live video input (webcam) to be mixed with the
contents of a web page and streamed to a [Janus WebRTC server](https://janus.conf.meetecho.com).


## Installation

No binary package is provided for this demo yet. For the time being you need to
build it from source.

The pre-requirements on the publisher side are:

- NodeJS
- Rust
- GStreamer (including gst-plugins-bad with wpesrc enabled)

Then you need a Janus instance, running on a remote server. This Janus instance
should have the video-room plugin enabled and the WebSocket transport plugin
enabled. You might need to open the TCP port 8989 and some UDP ports as well, as
required for RTP. Then copy the contents of the janus-web-app to the server HTTP
htdocs directory. We will assume the location of the Janus instance is
https://janus.example.com.

The next step is to build the publisher app:

1. Ensure you have Rust/Cargo installed with [rustup](https://rustup.rs)
2. Make sure you have GstWPE available on your Linux machine. It is provided by
   gst-plugins-bad since version 1.16. This command should show the details of
   the plugin: `gst-inspect-1.0 wpesrc`.
3. Compile the Rust app: `cargo build --release`
4. Start the Node app: `npm i wpe-graphics-overlays; node wpe-graphics-overlays/server.js`
5. Open the admin web-ui located at http://127.0.0.1:3000/admin
6. Start the Rust app: `cargo run --release -- -s wss://janus.example.com:8989 -r 1234 -f 42`

So the app will connect to Janus over WebSockets and hopefully publish the live
stream in the room 1234, with a feed ID of 42. These values are also referenced
in the webrtc.js file of the janus-web-app.

You should also see a GTK window popup on your desktop, showing the video
preview. This could be made optional though.

Finally, more clients can connect to the janus-web-app, to watch the live stream.

## Further configuration

By default the publisher app will encode the video in VP8. You can switch to VP9
or H264 by editing the
`~/.config/com.igalia.gstwpe.broadcast.demo/settings.toml` file. You can also
change the video resolution there.

## Credits

The code is adapted from the [RustFest 2019 Barcelona Rust/GTK/GStreamer workshop app](https://github.com/sdroege/rustfest-barcelona19-gst-webrtc-workshop). Many thanks to Sebastian Dröge <sebastian@centricular.com>!

The HTML/CSS template is based on the [Pure CSS Horizontal Ticker codepen](https://codepen.io/lewismcarey/pen/GJZVoG).

The NodeJS app is a fork of the [Roses CasparCG
Graphics](https://github.com/moschopsuk/Roses-2015-CasparCG-Graphics) authored
by Luke Moscrop.

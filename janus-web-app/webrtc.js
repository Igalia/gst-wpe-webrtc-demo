var remoteVideo;
var peerConnection;
var janusConnection;
var sessionId;
var handleId;
const roomId = 1234;
const feedId = 42;
const CONFIG = { audio: false, video: false, iceServers: [ {urls: 'stun:stun.l.google.com:19302'}, ] };
const RECEIVERS = { create_session, create_handle, publish, join_subscriber, ack };

function send(msg) {
    janusConnection.send(JSON.stringify(msg));
}

function ack(payload) {}

function start() {
    remoteVideo = document.getElementById('remoteVideo');
    janusConnection = new WebSocket('wss://' + window.location.hostname + ':8989', 'janus-protocol');
    janusConnection.onmessage = function(message) {
        var payload = JSON.parse(message.data);
        var receiver = RECEIVERS[payload.janus] || RECEIVERS[payload.transaction] || console.log;
        receiver(payload);
    };
    janusConnection.onopen = function(event) {
        send({janus: 'create', transaction: 'create_session'});
    };
}

function join_broadcast() {
    peerConnection = new RTCPeerConnection(CONFIG);
    peerConnection.onicecandidate = on_ice_candidate;
    peerConnection.ontrack = on_track;

    send({janus: 'message', transaction: 'join_subscriber',
          body: {request : 'join', ptype: 'subscriber', room: roomId, feed: feedId},
          session_id: sessionId, handle_id: handleId});
}

function on_ice_candidate(event) {
    send({janus: 'trickle', transaction: 'candidate', candidate: event.candidate,
          session_id: sessionId, handle_id: handleId});
}

function on_track(event) {
    remoteVideo.srcObject = event.streams[0];
    remoteVideo.play();
}

function keepalive() {
    send({janus: 'keepalive', transaction: 'keepalive', session_id: sessionId});
}

function create_session(payload) {
    sessionId = payload.data.id;
    setInterval(keepalive, 30000);
    send({janus: 'attach', transaction: 'create_handle', plugin: 'janus.plugin.videoroom', session_id: sessionId});
}

function create_handle(payload) {
    handleId = payload.data.id;
    join_broadcast();
}

function publish(payload) {
    peerConnection.setRemoteDescription(new RTCSessionDescription(payload.jsep));
}

function join_subscriber(payload) {
    if (!payload.jsep) {
        var container = document.getElementById('message');
        if (payload.plugindata.data.error_code == 428) {
            container.innerHTML = "GstWPE demo is offline. ";
        }
        container.innerHTML += payload.plugindata.data.error;
        return;
    }
    peerConnection.setRemoteDescription(new RTCSessionDescription(payload.jsep));
    peerConnection.createAnswer().then(function(answer) {
        peerConnection.setLocalDescription(answer);
        send({janus: 'message', transaction: 'blah', body: {request: 'start'},
              jsep: answer, session_id: sessionId, handle_id: handleId});
    });
}

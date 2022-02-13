import { Component } from "react";
import "./styles/App.scss";

import Header from "./components/Header";
import Messages from "./components/Messages";
import MessageBar from "./components/MessageBar";
import Participants from "./components/Participants";
import SignUp from "./components/SignUp";
import { MessageProps, MessageType } from "./components/Message";

interface AppState {
  id: number | null,
  phase: AppPhase,
  messages: MessageProps[],
  participants: string[] | null,
}

enum AppPhase {
  SignUp,
  Connecting,
  Connected
}

export class App extends Component<{}, AppState> {
  ws: WebSocket | null;

  constructor(props: {}) {
    super(props);

    this.ws = null;

    this.state = {
      id: null,
      phase: AppPhase.SignUp,
      messages: [],
      participants: null
    }

    this.connect = this.connect.bind(this);
    this.recvId = this.recvId.bind(this);
    this.recvMessage = this.recvMessage.bind(this);
    this.recvChatMessage = this.recvChatMessage.bind(this);
    this.recvParticipantUpdateMessage = this.recvParticipantUpdateMessage.bind(this);
    this.sendMessage = this.sendMessage.bind(this);
  }

  connect(username: string) {
    this.setState({ phase: AppPhase.Connecting });

    this.ws = new WebSocket("ws://localhost/ws");
    this.ws.onmessage = e => this.recvId(e, username);
  }

  async recvId(e: MessageEvent, username: string) {
    let buf = await e.data.arrayBuffer();
    let view = new DataView(buf, 4, 4);
    let id = view.getUint32(0, false);

    this.setState({ id, phase: AppPhase.Connected });
    this.ws!.onmessage = this.recvMessage;
    this.ws!.send(username);
  }

  async recvMessage(e: MessageEvent) {
    let buf: ArrayBuffer = await e.data.arrayBuffer();

    let typeView = new DataView(buf, 0, 1);
    let messageType = typeView.getUint8(0);

    if (messageType === 0) this.recvChatMessage(buf.slice(1));
    else if (messageType === 1) this.recvParticipantUpdateMessage(buf.slice(1));
  }

  recvChatMessage(buf: ArrayBuffer) {
    let messageLengthView = new DataView(buf, 4, 4);
    let messageLength = messageLengthView.getUint32(0, false);
    let messageView = Array.from(new Uint8Array(buf, 8, messageLength));
    let message = String.fromCharCode(...messageView);

    let senderIdView = new DataView(buf, 12 + messageLength, 4);
    let senderId = senderIdView.getUint32(0, false);

    let sender = "";
    if (senderId !== 0) {
      let senderLengthView = new DataView(buf, 20 + messageLength, 4);
      let senderLength = senderLengthView.getUint32(0, false);
      let senderView = Array.from(new Uint8Array(buf, 24 + messageLength, senderLength));
      sender = String.fromCharCode(...senderView);
    }

    let messages = this.state.messages;
    let timestamp = new Date().getTime();

    let type = MessageType.Remote;
    if (senderId === this.state.id) type = MessageType.Local;
    if (senderId === 0) type = MessageType.Broadcast;

    messages.push({
      message,
      sender,
      timestamp,
      type
    });

    this.setState({ messages });
  }

  recvParticipantUpdateMessage(buf: ArrayBuffer) {
    let participantsLengthView = new DataView(buf, 4, 4);
    let participantsLength = participantsLengthView.getUint32(0, false);

    let participantsView = Array.from(new Uint8Array(buf, 8, participantsLength - 1));
    let participants = String.fromCharCode(...participantsView).split("\n");

    this.setState({ participants });
  }

  sendMessage(message: string) {
    this.ws!.send(message);
  }

  render() {
    if (this.state.phase === AppPhase.SignUp || this.state.phase === AppPhase.Connecting) {
      return <SignUp
        onSignUp={this.connect}
        loading={this.state.phase === AppPhase.Connecting} />;
    } else {
      return (
        <div className="App">
          <Header />
          <Messages messages={this.state.messages} />
          <Participants participants={this.state.participants} />
          <MessageBar onSendMessage={this.sendMessage} />
        </div>
      )
    }
  }
}

export default App;
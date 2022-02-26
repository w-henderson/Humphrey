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

enum ClientMessageKind {
  Register,
  Chat
}

enum ServerMessageKind {
  Id,
  Participants,
  Join,
  Chat,
  Leave
}

interface ClientMessage {
  kind: ClientMessageKind,
  message: string
}

interface ServerMessage {
  kind: ServerMessageKind,
  message: string | null,
  senderId: number,
  senderName: string | null,
  participants?: string[]
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
    this.sendMessage = this.sendMessage.bind(this);
  }

  connect(username: string) {
    this.setState({ phase: AppPhase.Connecting });

    let websocketAddr = (window.location.protocol === "https:" ? "wss" : "ws")
      + `://${window.location.host}/ws`;

    this.ws = new WebSocket(websocketAddr);
    this.ws.onmessage = e => this.recvId(e, username);
  }

  recvId(e: MessageEvent, username: string) {
    let incomingMessage: ServerMessage = JSON.parse(e.data);

    let message: ClientMessage = {
      kind: ClientMessageKind.Register,
      message: username
    };

    this.setState({ id: incomingMessage.senderId, phase: AppPhase.Connected });
    this.ws!.onmessage = this.recvMessage;
    this.ws!.send(JSON.stringify(message));
  }

  recvMessage(e: MessageEvent) {
    let incomingMessage: ServerMessage = JSON.parse(e.data);

    switch (incomingMessage.kind) {
      case ServerMessageKind.Participants: {
        return this.setState({ participants: incomingMessage.participants! });
      }
      case ServerMessageKind.Join: {
        let participants = this.state.participants;
        let messages = this.state.messages;

        participants?.push(incomingMessage.senderName!);

        messages.push({
          message: `${incomingMessage.senderName!} has joined the chat`,
          sender: "",
          timestamp: new Date().getTime(),
          type: MessageType.Broadcast
        });

        return this.setState({ participants, messages });
      }
      case ServerMessageKind.Chat: {
        let messages = this.state.messages;

        messages.push({
          message: incomingMessage.message!,
          sender: incomingMessage.senderName!,
          timestamp: new Date().getTime(),
          type: incomingMessage.senderId === this.state.id ? MessageType.Local : MessageType.Remote
        });

        return this.setState({ messages });
      }
      case ServerMessageKind.Leave: {
        let participants = this.state.participants;
        let messages = this.state.messages;

        participants!.splice(participants!.indexOf(incomingMessage.senderName!), 1);

        messages.push({
          message: `${incomingMessage.senderName} has left the chat`,
          sender: "",
          timestamp: new Date().getTime(),
          type: MessageType.Broadcast
        });

        return this.setState({ participants, messages });
      }
    }
  }

  sendMessage(message: string) {
    let serialized: ClientMessage = {
      kind: ClientMessageKind.Chat,
      message
    }

    this.ws!.send(JSON.stringify(serialized));
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
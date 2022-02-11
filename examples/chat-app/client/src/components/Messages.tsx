import { Component } from "react";
import "../styles/Messages.scss";
import Message, { MessageType } from "./Message";

export class Messages extends Component {
  render() {
    return (
      <div className="Messages">
        <Message
          message="Hello, world!"
          sender="Humphrey"
          timestamp={1234567890123}
          type={MessageType.Broadcast} />
        <Message
          message="Hello, world!"
          sender="Humphrey"
          timestamp={1234567890123}
          type={MessageType.Remote} />
        <Message
          message="Hello, world!"
          sender="Humphrey"
          timestamp={1234567890123}
          type={MessageType.Local} />
      </div>
    )
  }
}

export default Messages;
import { Component } from "react";
import "../styles/Messages.scss";
import Message, { MessageProps, MessageType } from "./Message";

interface MessagesProps {
  messages: MessageProps[],
}

export class Messages extends Component<MessagesProps> {
  render() {
    return (
      <div className="Messages">
        {this.props.messages.map((message, index) =>
          <Message
            key={index}
            {...message} />
        )}
      </div>
    )
  }
}

export default Messages;
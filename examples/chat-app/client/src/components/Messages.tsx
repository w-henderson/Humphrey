import React, { Component } from "react";
import "../styles/Messages.scss";
import Message, { MessageProps } from "./Message";

interface MessagesProps {
  messages: MessageProps[],
}

export class Messages extends Component<MessagesProps> {
  div: React.RefObject<HTMLDivElement>;

  constructor(props: MessagesProps) {
    super(props);

    this.div = React.createRef();
  }

  componentDidUpdate() {
    this.div.current!.scrollTop = this.div.current!.scrollHeight;
  }

  render() {
    return (
      <div className="Messages" ref={this.div}>
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
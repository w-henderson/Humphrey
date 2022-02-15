import React, { Component } from "react";
import "../styles/MessageBar.scss";

interface MessageBarProps {
  onSendMessage: (message: string) => void,
}

interface MessageBarState {
  message: string
}

export class MessageBar extends Component<MessageBarProps, MessageBarState> {
  inputRef: React.RefObject<HTMLInputElement>;

  constructor(props: MessageBarProps) {
    super(props);

    this.state = {
      message: ""
    }

    this.inputRef = React.createRef();

    this.preSend = this.preSend.bind(this);
  }

  componentDidMount() {
    this.inputRef.current!.focus();
  }

  preSend() {
    let message = this.state.message.trim();
    if (message.length > 0) {
      this.setState({ message: "" });
      this.props.onSendMessage(message);
    }
  }

  render() {
    return (
      <div className="MessageBar">
        <input
          placeholder="Type a message"
          onChange={(e) => this.setState({ message: e.target.value })}
          onKeyDown={(e) => { if (e.key === "Enter") this.preSend() }}
          value={this.state.message}
          ref={this.inputRef} />

        <i
          className={`bi bi-cursor`}
          onClick={this.preSend} />
      </div>
    )
  }
}

export default MessageBar;
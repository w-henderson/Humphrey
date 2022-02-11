import { Component } from "react";
import "../styles/Messages.scss";
import Message from "./Message";

export class Messages extends Component {
  render() {
    return (
      <div className="Messages">
        <Message />
      </div>
    )
  }
}

export default Messages;
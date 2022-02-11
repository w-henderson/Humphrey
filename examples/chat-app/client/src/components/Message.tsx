import { Component } from "react";
import "../styles/Message.scss";

export class Message extends Component {
  render() {
    return (
      <div className="Message senderLocal">
        <div className="messageTime">
          12:34
        </div>
        <div className="messageContent">
          Hello, world!
        </div>
      </div>
    )
  }
}

export default Message;
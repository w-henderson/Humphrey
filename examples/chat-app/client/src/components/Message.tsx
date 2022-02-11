import { Component } from "react";
import humanize from "../DateHumanizer";
import "../styles/Message.scss";

interface MessageProps {
  message: string,
  sender: string,
  type: MessageType,
  timestamp: number,
}

export enum MessageType {
  Local,
  Remote,
  Broadcast
}

export class Message extends Component<MessageProps> {
  render() {
    if (this.props.type === MessageType.Local) {
      return (
        <div className="Message senderLocal">
          <div className="messageTime">
            {humanize(new Date(this.props.timestamp))}
          </div>
          <div className="messageContent">
            {this.props.message}
          </div>
        </div>
      )
    } else if (this.props.type === MessageType.Remote) {
      return (
        <div className="Message senderRemote">
          <div className="messageContent">
            <div className="messageSender">
              {this.props.sender}
            </div>

            {this.props.message}
          </div>
          <div className="messageTime">
            {humanize(new Date(this.props.timestamp))}
          </div>
        </div>
      )
    } else {
      return (
        <div className="Message broadcast">
          <div className="messageTime">
            {humanize(new Date(this.props.timestamp))}
          </div>
          <div className="messageContent">
            {this.props.message}
          </div>
        </div>
      )
    }
  }
}

export default Message;
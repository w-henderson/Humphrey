import { Component } from "react";
import "../styles/MessageBar.scss";

export class MessageBar extends Component {
  render() {
    return (
      <div className="MessageBar">
        <input
          placeholder="Type a message"
          onChange={(e) => this.setState({ message: e.target.value })}
          //onKeyDown={this.preSend}
          /*value={this.state.message}*/ />

        <i
          className={`bi bi-cursor`} />
      </div>
    )
  }
}

export default MessageBar;
import { Component } from "react";
import "../styles/Participants.scss";

interface ParticipantsProps {
  participants: string[] | null,
}

export class Participants extends Component<ParticipantsProps> {
  render() {
    if (this.props.participants === null || this.props.participants.length === 0) {
      return (
        <div className="Participants">
          <h2>Humphrey Chat</h2>
          Welcome to Humphrey Chat, an example real-time chat application built with Humphrey and Humphrey WebSocket. The backend is written in Rust using Humphrey and no other dependencies, and the frontend is written in TypeScript using React. You can find the source code for Humphrey Chat on <a href="https://github.com/w-henderson/Humphrey/tree/master/examples/chat-app" target="_blank">GitHub</a>.

          <h2>Participants</h2>

          Loading...
        </div>
      )
    } else {
      return (
        <div className="Participants">
          <h2>Humphrey Chat</h2>
          Welcome to Humphrey Chat, an example real-time chat application built with Humphrey and Humphrey WebSocket. The backend is written in Rust using Humphrey and no other dependencies, and the frontend is written in TypeScript using React. You can find the source code for Humphrey Chat on <a href="https://github.com/w-henderson/Humphrey/tree/master/examples/chat-app" target="_blank">GitHub</a>.

          <h2>Participants</h2>

          <div className="container">
            {this.props.participants.map((participant, index) =>
              <div key={index}>
                <i className="bi bi-person-fill" />
                <span>{participant}</span>
              </div>
            )}
          </div>
        </div>
      )
    }
  }
}

export default Participants;
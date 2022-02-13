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
          <h2>Participants</h2>

          Loading...
        </div>
      )
    } else {
      return (
        <div className="Participants">
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
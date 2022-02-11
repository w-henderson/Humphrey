import { Component } from "react";
import "../styles/Participants.scss";

export class Participants extends Component {
  render() {
    return (
      <div className="Participants">
        <h2>Participants</h2>

        <div className="container">
          <div>
            <i className="bi bi-person-fill" />
            <span>Humphrey</span>
          </div>

          <div>
            <i className="bi bi-person-fill" />
            <span>Humphrey</span>
          </div>

          <div>
            <i className="bi bi-person-fill" />
            <span>Humphrey</span>
          </div>

          <div>
            <i className="bi bi-person-fill" />
            <span>Humphrey</span>
          </div>
        </div>
      </div>
    )
  }
}

export default Participants;
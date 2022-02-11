import { Component } from "react";
import "../styles/SignUp.scss";

interface SignUpProps {
  onSignUp: (username: string) => void,
  loading: boolean
}

interface SignUpState {
  username: string
}

class SignUp extends Component<SignUpProps, SignUpState> {
  constructor(props: SignUpProps) {
    super(props);

    this.state = {
      username: ""
    }
  }

  render() {
    if (!this.props.loading) {
      return (
        <div className="SignUp">
          <h1>Humphrey Chat</h1>

          <input
            type="text"
            placeholder="Choose a username"
            value={this.state.username}
            onChange={(e) => this.setState({ username: e.target.value })} />

          <input
            type="button"
            value="Join Chat"
            onClick={() => this.props.onSignUp(this.state.username)} />
        </div>
      )
    } else {
      return (
        <div className="SignUp loading">
          <i
            className="bi bi-arrow-clockwise" />
        </div>
      )
    }
  }
}

export default SignUp;
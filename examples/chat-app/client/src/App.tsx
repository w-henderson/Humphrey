import { Component } from "react";
import "./styles/App.scss";

import Header from "./components/Header";
import Messages from "./components/Messages";
import MessageBar from "./components/MessageBar";
import Participants from "./components/Participants";

export class App extends Component {
  render() {
    return (
      <div className="App">
        <Header />
        <Messages />
        <Participants />
        <MessageBar />
      </div>
    )
  }
}

export default App;
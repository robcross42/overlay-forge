import React from "react";
import ReactDOM from "react-dom/client";
import App from "./app/App";
import GameChatOverlayApp from "./app/GameChatOverlayApp";
import "./styles/global.css";

const params = new URLSearchParams(window.location.search);
const RootApp = params.get("window") === "game-chat" ? GameChatOverlayApp : App;

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <RootApp />
  </React.StrictMode>
);

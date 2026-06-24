import React from "react";
import ReactDOM from "react-dom/client";
import App from "./app/App";
import GameBuildGuideOverlayApp from "./app/GameBuildGuideOverlayApp";
import GameChatOverlayApp from "./app/GameChatOverlayApp";
import "./styles/global.css";

const params = new URLSearchParams(window.location.search);
const windowMode = params.get("window");
const RootApp =
  windowMode === "game-chat"
    ? GameChatOverlayApp
    : windowMode === "game-build-guide"
      ? GameBuildGuideOverlayApp
      : App;

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <RootApp />
  </React.StrictMode>
);

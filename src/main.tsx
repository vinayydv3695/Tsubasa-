import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles/globals.css";

// Apply theme immediately to prevent flash of wrong theme
const savedTheme = localStorage.getItem("tsubasa-theme");
const theme = savedTheme === "rose" || savedTheme === "light" ? savedTheme : "black";
document.documentElement.setAttribute("data-theme", theme);

const rootEl = document.getElementById("root");
if (!rootEl) throw new Error("Root element not found");

ReactDOM.createRoot(rootEl).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);

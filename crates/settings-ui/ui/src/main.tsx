import { createRoot } from "react-dom/client";
import App from "./App";
import "./styles.css";

const container = document.getElementById("root");
if (!container) {
  throw new Error("Missing #root element");
}

// StrictMode is intentionally not used: the settings window should run
// bootstrap()/show_window() exactly once on mount.
createRoot(container).render(<App />);

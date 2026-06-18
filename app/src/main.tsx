import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { App } from "./ui/App";
import { AppStoreProvider } from "./store/appStore";
import "./styles/main.scss";

const root = document.querySelector("#root");

if (!root) {
  throw new Error("Missing #root element");
}

createRoot(root).render(
  <StrictMode>
    <AppStoreProvider>
      <App />
    </AppStoreProvider>
  </StrictMode>,
);

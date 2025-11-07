import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import '@mantine/core/styles.css';
import '@mantine/notifications/styles.css';
import { MantineProvider } from '@mantine/core';
import { Notifications } from '@mantine/notifications';
import { theme } from './theme';
import { WorkspaceProvider } from './context/WorkspaceContext';
import { SessionProvider } from './context/SessionContext';

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <MantineProvider theme={theme}>
      <Notifications position="top-right" />
      <WorkspaceProvider>
        <SessionProvider>
          <App />
        </SessionProvider>
      </WorkspaceProvider>
    </MantineProvider>
  </React.StrictMode>,
);

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { HashRouter, Routes, Route } from "react-router-dom";
import "@vscode/codicons/dist/codicon.css";
import "./index.css";
import { HostContext, createBrowserHost } from "./host";
import { Layout } from "./Layout";
import { DashboardPage } from "./routes/DashboardPage";
import { ConfigPage } from "./routes/ConfigPage";
import { IssueTypesPage } from "./routes/IssueTypesPage";
import { QueuePage } from "./routes/QueuePage";
import { StatusPage } from "./routes/StatusPage";
import { SectionPage } from "./routes/SectionPage";
import { AgentDetailPage } from "./routes/AgentDetailPage";
import { ModelProvidersPage } from "./routes/ModelProvidersPage";
import { LoginPage } from "./routes/LoginPage";
import { ForgotPasswordPage } from "./routes/ForgotPasswordPage";
import { ResetPasswordPage } from "./routes/ResetPasswordPage";
import { SetupPage } from "./routes/SetupPage";
import { DevicePage } from "./routes/DevicePage";
import { SecurityPage } from "./routes/SecurityPage";
import { OnboardingPage } from "./routes/onboarding/OnboardingPage";
import { WorkspaceGate } from "./WorkspaceGate";
import { ProfilesProvider } from "./profiles-context";
import { LicensePage } from "./routes/LicensePage";
import { RemoteTargetsPage } from "./routes/RemoteTargetsPage";

const host = createBrowserHost();

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <HostContext.Provider value={host}>
      <HashRouter>
        <Routes>
          {/* Unauthenticated screens render outside Layout: the shell's own
              API calls would 401 for a visitor who cannot yet authenticate. */}
          <Route path="login" element={<LoginPage />} />
          <Route path="forgot-password" element={<ForgotPasswordPage />} />
          <Route path="reset-password" element={<ResetPasswordPage />} />
          <Route path="setup" element={<SetupPage />} />
          <Route element={<ProfilesProvider />}>
            <Route path="onboarding" element={<OnboardingPage />} />
            <Route element={<WorkspaceGate />}>
              <Route element={<Layout />}>
                <Route path="device" element={<DevicePage />} />
                <Route path="settings/security" element={<SecurityPage />} />
                <Route path="settings/license" element={<LicensePage />} />
                <Route path="remote-targets" element={<RemoteTargetsPage />} />
                <Route index element={<DashboardPage />} />
                <Route path="config" element={<ConfigPage />} />
                <Route path="connections" element={<SectionPage conceptKey="connections" />} />
                <Route path="kanban" element={<SectionPage conceptKey="kanban" />} />
                <Route path="llm" element={<SectionPage conceptKey="llm" />} />
                <Route path="model-providers" element={<ModelProvidersPage />} />
                <Route path="git" element={<SectionPage conceptKey="git" />} />
                <Route path="issuetypes" element={<IssueTypesPage />} />
                <Route path="delegators" element={<SectionPage conceptKey="delegators" />} />
                <Route path="projects" element={<SectionPage conceptKey="projects" />} />
                <Route path="workflows" element={<SectionPage conceptKey="workflows" />} />
                <Route path="queue" element={<QueuePage />} />
                <Route path="status" element={<StatusPage />} />
                <Route path="agent/:id" element={<AgentDetailPage />} />
              </Route>
            </Route>
          </Route>
        </Routes>
      </HashRouter>
    </HostContext.Provider>
  </StrictMode>,
);

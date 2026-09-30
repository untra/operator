import { Navigate, Outlet } from "react-router-dom";
import { useApiQuery } from "./api";
import { setupStatusQuery } from "./api/definitions";

export function WorkspaceGate() {
  const { data, isLoading } = useApiQuery(setupStatusQuery());

  if (isLoading || !data) {
    return null;
  }
  return data.initialized ? <Outlet /> : <Navigate to="/onboarding" replace />;
}

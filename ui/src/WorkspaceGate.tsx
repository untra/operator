import { useEffect, useState } from 'react';
import { Navigate, Outlet } from 'react-router-dom';
import { OperatorApi } from './api-client';
import { useHost } from './host';

export function WorkspaceGate() {
  const host = useHost();
  const [initialized, setInitialized] = useState<boolean | null>(null);

  useEffect(() => {
    let active = true;
    new OperatorApi(host)
      .setupStatus()
      .then((status) => active && setInitialized(status.initialized))
      .catch(() => active && setInitialized(null));
    return () => { active = false; };
  }, [host]);

  if (initialized === null) {
    return null;
  }
  return initialized ? <Outlet /> : <Navigate to="/onboarding" replace />;
}

import { useEffect, useState } from 'react';

interface HealthResponse {
  status: string;
  version: string;
}

export function App() {
  const [health, setHealth] = useState<HealthResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetch('/api/v1/health')
      .then((res) => res.json())
      .then((data) => setHealth(data))
      .catch((err) => setError(err.message));
  }, []);

  return (
    <div style={{ fontFamily: 'system-ui, sans-serif', padding: '2rem' }}>
      <h1>Operator</h1>
      {error && <p style={{ color: '#e55' }}>API: {error}</p>}
      {health && (
        <p style={{ color: '#5a5' }}>
          API: {health.status} (v{health.version})
        </p>
      )}
      {!health && !error && <p>Connecting to API...</p>}
    </div>
  );
}

import React from 'react';
import { createRoot } from 'react-dom/client';
import '../images/icons/dist/operator-icons.css';
import { App } from './App';

const container = document.getElementById('root');
if (container) {
  const root = createRoot(container);
  root.render(<App />);
}

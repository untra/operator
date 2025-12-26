/**
 * Operator Backstage Backend
 *
 * Minimal Bun-based backend for local catalog browsing.
 * Includes proxy for Operator API integration.
 */

import { createBackend } from '@backstage/backend-defaults';

const backend = createBackend();

// Core plugins
backend.add(import('@backstage/plugin-app-backend'));
backend.add(import('@backstage/plugin-catalog-backend'));

// Proxy for Operator REST API
backend.add(import('@backstage/plugin-proxy-backend'));

// Auth with guest provider
backend.add(import('@backstage/plugin-auth-backend'));
backend.add(import('@backstage/plugin-auth-backend-module-guest-provider'));

// Start the backend
backend.start();

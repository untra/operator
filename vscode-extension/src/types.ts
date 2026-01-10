/**
 * Type re-exports from generated bindings
 *
 * This file provides backwards-compatible type aliases for the generated
 * TypeScript types from Rust. The actual type definitions come from
 * ../bindings/ via the copy-types.js prebuild script.
 */

// TicketType is now a dynamic string (any uppercase key like "FEAT", "BUG", "CUSTOM")
export type TicketType = string;

// Re-export generated types with original names for backwards compatibility
export {
  VsCodeTicketStatus as TicketStatus,
  VsCodeTicketInfo as TicketInfo,
  VsCodeTerminalCreateOptions as TerminalCreateOptions,
  VsCodeTerminalState as TerminalState,
  VsCodeActivityState as ActivityState,
  VsCodeHealthResponse as HealthResponse,
  VsCodeSuccessResponse as SuccessResponse,
  VsCodeExistsResponse as ExistsResponse,
  VsCodeActivityResponse as ActivityResponse,
  VsCodeListResponse as ListResponse,
  VsCodeErrorResponse as ErrorResponse,
  VsCodeSendCommandRequest as SendCommandRequest,
  VsCodeSessionInfo as SessionInfo,
  VsCodeModelOption as ModelOption,
  VsCodeLaunchOptions as LaunchOptions,
  VsCodeTicketMetadata as TicketMetadata,
} from './generated';

// Also export the VsCode-prefixed versions for direct use
export {
  VsCodeTicketStatus,
  VsCodeTicketInfo,
  VsCodeTerminalCreateOptions,
  VsCodeTerminalState,
  VsCodeActivityState,
  VsCodeHealthResponse,
  VsCodeSuccessResponse,
  VsCodeExistsResponse,
  VsCodeActivityResponse,
  VsCodeListResponse,
  VsCodeErrorResponse,
  VsCodeSendCommandRequest,
  VsCodeSessionInfo,
  VsCodeModelOption,
  VsCodeLaunchOptions,
  VsCodeTicketMetadata,
} from './generated';

// Export issue type metadata (for dynamic styling)
export { IssueTypeSummary } from './generated';

// Export REST API types
export {
  LaunchTicketRequest,
  LaunchTicketResponse,
  LlmProvider,
} from './generated';

/**
 * Ticket metadata parser for YAML frontmatter
 *
 * Parses ticket markdown files to extract session IDs and other
 * metadata stored in YAML frontmatter.
 */

import * as fs from 'fs/promises';
import { TicketMetadata } from './types';

/**
 * Parse YAML frontmatter from ticket markdown file
 */
export async function parseTicketMetadata(
  filePath: string
): Promise<TicketMetadata | null> {
  try {
    const content = await fs.readFile(filePath, 'utf-8');
    return parseTicketContent(content);
  } catch {
    return null;
  }
}

/**
 * Parse ticket content string (for testing)
 */
export function parseTicketContent(content: string): TicketMetadata | null {
  // Extract YAML frontmatter between --- markers
  const match = content.match(/^---\n([\s\S]*?)\n---/);
  if (!match) {
    return null;
  }

  const yaml = match[1];
  const metadata: TicketMetadata = {
    id: '',
    status: '',
    step: '',
    priority: '',
    project: '',
  };

  // Simple YAML parsing for known fields
  for (const line of yaml.split('\n')) {
    // Skip empty lines and lines that start with whitespace (nested)
    if (!line.trim() || line.startsWith(' ') || line.startsWith('\t')) {
      continue;
    }

    const colonIndex = line.indexOf(':');
    if (colonIndex === -1) {
      continue;
    }

    const key = line.slice(0, colonIndex).trim();
    const value = line.slice(colonIndex + 1).trim();

    switch (key) {
      case 'id':
        metadata.id = value;
        break;
      case 'status':
        metadata.status = value;
        break;
      case 'step':
        metadata.step = value;
        break;
      case 'priority':
        metadata.priority = value;
        break;
      case 'project':
        metadata.project = value;
        break;
      case 'worktree_path':
        metadata.worktreePath = value;
        break;
      case 'branch':
        metadata.branch = value;
        break;
    }
  }

  // Parse sessions block (indented key-value pairs under 'sessions:')
  const sessionsMatch = yaml.match(/sessions:\s*\n((?:\s{2}\S+:.*\n?)+)/);
  if (sessionsMatch) {
    metadata.sessions = {};
    for (const line of sessionsMatch[1].split('\n')) {
      const sessionMatch = line.match(/^\s+(\S+):\s*(.+)$/);
      if (sessionMatch) {
        metadata.sessions[sessionMatch[1]] = sessionMatch[2].trim();
      }
    }
  }

  return metadata;
}

/**
 * Get current session ID from ticket metadata
 *
 * Tries the current step first, then falls back to 'initial'
 */
export function getCurrentSessionId(
  metadata: TicketMetadata
): string | undefined {
  if (!metadata.sessions) {
    return undefined;
  }

  // Try current step first
  if (metadata.step && metadata.sessions[metadata.step]) {
    return metadata.sessions[metadata.step];
  }

  // Fall back to 'initial'
  return metadata.sessions['initial'];
}

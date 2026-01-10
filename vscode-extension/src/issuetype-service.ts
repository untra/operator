/**
 * Issue Type Service for Operator VS Code extension
 *
 * Provides dynamic issue type metadata (icons, colors, glyphs)
 * by fetching from the Operator REST API on startup.
 * Falls back to sensible defaults if API is unavailable.
 */

import * as vscode from 'vscode';
import { IssueTypeSummary } from './generated';

/**
 * Default issue types used when API is unavailable
 */
const DEFAULT_ISSUE_TYPES: IssueTypeSummary[] = [
  {
    key: 'FEAT',
    name: 'Feature',
    description: 'New functionality',
    mode: 'autonomous',
    glyph: '*',
    color: 'cyan',
    source: 'default',
    stepCount: 3,
  },
  {
    key: 'FIX',
    name: 'Bug Fix',
    description: 'Fix a bug',
    mode: 'autonomous',
    glyph: '#',
    color: 'red',
    source: 'default',
    stepCount: 3,
  },
  {
    key: 'TASK',
    name: 'Task',
    description: 'General task',
    mode: 'autonomous',
    glyph: '>',
    color: 'green',
    source: 'default',
    stepCount: 2,
  },
  {
    key: 'SPIKE',
    name: 'Spike',
    description: 'Research and investigation',
    mode: 'paired',
    glyph: '?',
    color: 'magenta',
    source: 'default',
    stepCount: 2,
  },
  {
    key: 'INV',
    name: 'Investigation',
    description: 'Investigate a failure',
    mode: 'paired',
    glyph: '!',
    color: 'yellow',
    source: 'default',
    stepCount: 2,
  },
];

/**
 * Map glyph characters to VSCode ThemeIcon names
 */
const GLYPH_TO_ICON: Record<string, string> = {
  '*': 'sparkle',
  '#': 'wrench',
  '>': 'tasklist',
  '?': 'beaker',
  '!': 'search',
  '+': 'add',
  '-': 'dash',
  '@': 'mention',
  '%': 'graph',
  '^': 'arrow-up',
  '&': 'link',
  '~': 'sync',
  '=': 'check',
};

/**
 * Map color names to VSCode ThemeColor references
 */
const COLOR_TO_THEME: Record<string, string> = {
  cyan: 'terminal.ansiCyan',
  red: 'terminal.ansiRed',
  green: 'terminal.ansiGreen',
  magenta: 'terminal.ansiMagenta',
  yellow: 'terminal.ansiYellow',
  blue: 'terminal.ansiBlue',
  white: 'terminal.ansiWhite',
  black: 'terminal.ansiBlack',
  brightCyan: 'terminal.ansiBrightCyan',
  brightRed: 'terminal.ansiBrightRed',
  brightGreen: 'terminal.ansiBrightGreen',
  brightMagenta: 'terminal.ansiBrightMagenta',
  brightYellow: 'terminal.ansiBrightYellow',
  brightBlue: 'terminal.ansiBrightBlue',
  brightWhite: 'terminal.ansiBrightWhite',
};

/**
 * Service for managing issue type metadata
 *
 * Fetches issue types from the Operator REST API and provides
 * lookup methods for icons, colors, and validation.
 */
export class IssueTypeService {
  private types: Map<string, IssueTypeSummary> = new Map();
  private outputChannel: vscode.OutputChannel;
  private baseUrl: string;

  constructor(outputChannel: vscode.OutputChannel, baseUrl?: string) {
    this.outputChannel = outputChannel;
    const config = vscode.workspace.getConfiguration('operator');
    this.baseUrl = baseUrl || config.get('apiUrl', 'http://localhost:7008');

    // Initialize with defaults
    this.loadDefaults();
  }

  /**
   * Load default issue types (used when API is unavailable)
   */
  private loadDefaults(): void {
    for (const type of DEFAULT_ISSUE_TYPES) {
      this.types.set(type.key, type);
    }
  }

  /**
   * Refresh issue types from the Operator REST API
   */
  async refresh(): Promise<void> {
    try {
      const response = await fetch(`${this.baseUrl}/api/v1/issuetypes`);

      if (!response.ok) {
        this.outputChannel.appendLine(
          `[IssueTypeService] Failed to fetch issue types: ${response.status}`
        );
        return;
      }

      const data = (await response.json()) as IssueTypeSummary[];

      // Clear and reload
      this.types.clear();
      for (const type of data) {
        this.types.set(type.key, type);
      }

      this.outputChannel.appendLine(
        `[IssueTypeService] Loaded ${data.length} issue types from API`
      );
    } catch (error) {
      // API not available - keep using defaults
      this.outputChannel.appendLine(
        `[IssueTypeService] API unavailable, using ${this.types.size} default types`
      );
    }
  }

  /**
   * Update the base URL for API calls
   */
  setBaseUrl(url: string): void {
    this.baseUrl = url;
  }

  /**
   * Get issue type metadata by key
   */
  getType(key: string): IssueTypeSummary | undefined {
    return this.types.get(key.toUpperCase());
  }

  /**
   * Get all known issue type keys
   */
  getKnownKeys(): string[] {
    return Array.from(this.types.keys());
  }

  /**
   * Check if a key is a known issue type
   */
  isKnownType(key: string): boolean {
    return this.types.has(key.toUpperCase());
  }

  /**
   * Get ThemeIcon for an issue type
   */
  getIcon(key: string): vscode.ThemeIcon {
    const type = this.getType(key);
    const glyph = type?.glyph ?? '?';
    const iconName = GLYPH_TO_ICON[glyph] ?? 'file';
    const color = this.getColor(key);
    return new vscode.ThemeIcon(iconName, color);
  }

  /**
   * Get icon name (without color) for an issue type
   */
  getIconName(key: string): string {
    const type = this.getType(key);
    const glyph = type?.glyph ?? '?';
    return GLYPH_TO_ICON[glyph] ?? 'file';
  }

  /**
   * Get ThemeColor for an issue type
   */
  getColor(key: string): vscode.ThemeColor | undefined {
    const type = this.getType(key);
    const colorName = type?.color;
    if (!colorName) {
      return undefined;
    }
    const themeColorId = COLOR_TO_THEME[colorName];
    return themeColorId ? new vscode.ThemeColor(themeColorId) : undefined;
  }

  /**
   * Extract ticket type from a ticket ID
   *
   * Parses the prefix before the first '-':
   * - "FEAT-123" -> "FEAT"
   * - "BUG-456" -> "BUG"
   * - "CUSTOM-789" -> "CUSTOM"
   * - "invalid" -> "TASK" (default)
   */
  extractTypeFromId(ticketId: string): string {
    const parts = ticketId.split('-');
    if (parts.length >= 2) {
      const prefix = parts[0].toUpperCase();
      // Validate it looks like a type key (uppercase letters only)
      if (/^[A-Z]+$/.test(prefix)) {
        return prefix;
      }
    }
    return 'TASK'; // Default fallback
  }

  /**
   * Parse ticket ID and type from a filename
   *
   * Supports formats:
   * - "FEAT-123.md" -> { id: "FEAT-123", type: "FEAT" }
   * - "FEAT-123-title.md" -> { id: "FEAT-123", type: "FEAT" }
   * - "random.md" -> { id: "random", type: "TASK" }
   */
  parseTicketFilename(filename: string): { id: string; type: string } {
    const baseName = filename.replace(/\.md$/, '');
    const match = baseName.match(/^([A-Z]+)-(\d+)/i);

    if (match) {
      const type = match[1].toUpperCase();
      const id = `${type}-${match[2]}`;
      return { id, type };
    }

    return { id: baseName, type: 'TASK' };
  }

  /**
   * Get icon for a terminal name (extracts type from name)
   */
  getIconForTerminal(name: string): vscode.ThemeIcon {
    // Terminal names are like "op-FEAT-123"
    const typeMatch = name.match(/op-([A-Z]+)-/i);
    if (typeMatch) {
      return this.getIcon(typeMatch[1]);
    }
    return new vscode.ThemeIcon('terminal');
  }

  /**
   * Get color for a terminal name (extracts type from name)
   */
  getColorForTerminal(name: string): vscode.ThemeColor {
    // Terminal names are like "op-FEAT-123"
    const typeMatch = name.match(/op-([A-Z]+)-/i);
    if (typeMatch) {
      return this.getColor(typeMatch[1]) ?? new vscode.ThemeColor('terminal.ansiWhite');
    }
    return new vscode.ThemeColor('terminal.ansiWhite');
  }
}

/**
 * Terminal lifecycle management for Operator VS Code extension
 *
 * Manages terminal creation, disposal, and activity tracking.
 * Terminals are styled by ticket type with colors and icons.
 */

import * as vscode from 'vscode';
import { TerminalCreateOptions, TerminalState, ActivityState } from './types';

/**
 * Manages operator terminals with activity detection and styling
 */
export class TerminalManager {
  private terminals = new Map<string, vscode.Terminal>();
  private activityState = new Map<string, ActivityState>();
  private createdAt = new Map<string, number>();
  private disposables: vscode.Disposable[] = [];

  constructor() {
    // Track shell execution for activity detection
    this.disposables.push(
      vscode.window.onDidStartTerminalShellExecution((e) => {
        const name = this.findTerminalName(e.terminal);
        if (name && this.terminals.has(name)) {
          this.activityState.set(name, 'running');
        }
      }),
      vscode.window.onDidEndTerminalShellExecution((e) => {
        const name = this.findTerminalName(e.terminal);
        if (name && this.terminals.has(name)) {
          this.activityState.set(name, 'idle');
        }
      }),
      vscode.window.onDidCloseTerminal((t) => {
        const name = this.findTerminalName(t);
        if (name) {
          this.terminals.delete(name);
          this.activityState.delete(name);
          this.createdAt.delete(name);
        }
      })
    );
  }

  /**
   * Create a new terminal with Operator styling
   */
  async create(options: TerminalCreateOptions): Promise<vscode.Terminal> {
    const { name, workingDir, env } = options;

    // Dispose existing terminal with same name if present
    if (this.terminals.has(name)) {
      await this.kill(name);
    }

    // Use ticket-specific colors and icons
    const terminalOptions: vscode.TerminalOptions = {
      name,
      cwd: workingDir,
      color: this.getColorForName(name),
      iconPath: this.getIconForName(name),
      env: {
        ...env,
        OPERATOR_SESSION: name,
      },
    };

    const terminal = vscode.window.createTerminal(terminalOptions);
    this.terminals.set(name, terminal);
    this.activityState.set(name, 'idle');
    this.createdAt.set(name, Date.now());

    return terminal;
  }

  /**
   * Send a command to a terminal
   */
  async send(name: string, command: string): Promise<void> {
    const terminal = this.terminals.get(name);
    if (!terminal) {
      throw new Error(`Terminal '${name}' not found`);
    }
    terminal.sendText(command);
  }

  /**
   * Reveal a terminal without taking focus (show in panel)
   */
  async show(name: string): Promise<void> {
    const terminal = this.terminals.get(name);
    if (!terminal) {
      throw new Error(`Terminal '${name}' not found`);
    }
    terminal.show(true); // preserveFocus = true
  }

  /**
   * Focus a terminal (takes keyboard focus)
   */
  async focus(name: string): Promise<void> {
    const terminal = this.terminals.get(name);
    if (!terminal) {
      throw new Error(`Terminal '${name}' not found`);
    }
    terminal.show(false); // preserveFocus = false
  }

  /**
   * Kill/dispose a terminal
   */
  async kill(name: string): Promise<void> {
    const terminal = this.terminals.get(name);
    if (terminal) {
      terminal.dispose();
      this.terminals.delete(name);
      this.activityState.delete(name);
      this.createdAt.delete(name);
    }
  }

  /**
   * Check if terminal exists
   */
  exists(name: string): boolean {
    return this.terminals.has(name);
  }

  /**
   * Get activity state
   */
  getActivity(name: string): ActivityState {
    return this.activityState.get(name) ?? 'unknown';
  }

  /**
   * List all managed terminals
   */
  list(): TerminalState[] {
    const result: TerminalState[] = [];

    for (const [name] of this.terminals) {
      result.push({
        name,
        pid: undefined, // processId is a Thenable, would need async handling
        activity: this.activityState.get(name) ?? 'unknown',
        createdAt: this.createdAt.get(name) ?? Date.now(),
      });
    }

    return result;
  }

  /**
   * Color scheme based on ticket type
   */
  private getColorForName(name: string): vscode.ThemeColor {
    // op-FEAT-123 -> cyan, op-FIX-123 -> red, etc.
    if (name.includes('FEAT')) {
      return new vscode.ThemeColor('terminal.ansiCyan');
    }
    if (name.includes('FIX')) {
      return new vscode.ThemeColor('terminal.ansiRed');
    }
    if (name.includes('TASK')) {
      return new vscode.ThemeColor('terminal.ansiGreen');
    }
    if (name.includes('SPIKE')) {
      return new vscode.ThemeColor('terminal.ansiMagenta');
    }
    if (name.includes('INV')) {
      return new vscode.ThemeColor('terminal.ansiYellow');
    }
    return new vscode.ThemeColor('terminal.ansiWhite');
  }

  /**
   * Icon based on ticket type
   */
  private getIconForName(name: string): vscode.ThemeIcon {
    if (name.includes('FEAT')) {
      return new vscode.ThemeIcon('sparkle');
    }
    if (name.includes('FIX')) {
      return new vscode.ThemeIcon('wrench');
    }
    if (name.includes('TASK')) {
      return new vscode.ThemeIcon('tasklist');
    }
    if (name.includes('SPIKE')) {
      return new vscode.ThemeIcon('beaker');
    }
    if (name.includes('INV')) {
      return new vscode.ThemeIcon('search');
    }
    return new vscode.ThemeIcon('terminal');
  }

  /**
   * Find terminal name by terminal instance
   */
  private findTerminalName(terminal: vscode.Terminal): string | undefined {
    for (const [name, t] of this.terminals) {
      if (t === terminal) {
        return name;
      }
    }
    return undefined;
  }

  /**
   * Dispose all resources
   */
  dispose(): void {
    this.disposables.forEach((d) => d.dispose());
    this.terminals.forEach((t) => t.dispose());
    this.terminals.clear();
    this.activityState.clear();
    this.createdAt.clear();
  }
}

import { createContext, useContext } from 'react';

export interface Host {
  baseUrl(): string;
  openExternal(url: string): void;
  browseFolder(): Promise<string | null>;
  openFile(path: string): void;
}

class BrowserHost implements Host {
  baseUrl(): string {
    return window.location.origin;
  }

  openExternal(url: string): void {
    window.open(url, '_blank');
  }

  async browseFolder(): Promise<string | null> {
    return null;
  }

  openFile(_path: string): void {
    // no-op in browser context
  }
}

export type VscodeApi = {
  postMessage(msg: unknown): void;
};

class VscodeHost implements Host {
  private vscode: VscodeApi;
  private apiUrl: string;

  constructor(vscode: VscodeApi, apiUrl: string) {
    this.vscode = vscode;
    this.apiUrl = apiUrl;
  }

  baseUrl(): string {
    return this.apiUrl;
  }

  openExternal(url: string): void {
    this.vscode.postMessage({ type: 'openExternal', url });
  }

  async browseFolder(): Promise<string | null> {
    return new Promise((resolve) => {
      const handler = (event: MessageEvent) => {
        if (event.data?.type === 'browseResult') {
          window.removeEventListener('message', handler);
          resolve(event.data.path ?? null);
        }
      };
      window.addEventListener('message', handler);
      this.vscode.postMessage({ type: 'browseFolder', field: 'workingDirectory' });
    });
  }

  openFile(filePath: string): void {
    this.vscode.postMessage({ type: 'openFile', filePath });
  }
}

export function createBrowserHost(): Host {
  return new BrowserHost();
}

export function createVscodeHost(vscode: VscodeApi, apiUrl: string): Host {
  return new VscodeHost(vscode, apiUrl);
}

export const HostContext = createContext<Host>(new BrowserHost());

export function useHost(): Host {
  return useContext(HostContext);
}

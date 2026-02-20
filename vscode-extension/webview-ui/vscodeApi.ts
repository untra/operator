import type { WebviewToExtensionMessage, ExtensionToWebviewMessage } from './types/messages';

interface VSCodeApi {
  postMessage(message: WebviewToExtensionMessage): void;
  getState(): unknown;
  setState(state: unknown): void;
}

declare function acquireVsCodeApi(): VSCodeApi;

let _api: VSCodeApi | undefined;

function getApi(): VSCodeApi {
  if (!_api) {
    _api = acquireVsCodeApi();
  }
  return _api;
}

export function postMessage(message: WebviewToExtensionMessage): void {
  getApi().postMessage(message);
}

export function onMessage(
  handler: (message: ExtensionToWebviewMessage) => void
): () => void {
  const listener = (event: MessageEvent<ExtensionToWebviewMessage>) => {
    handler(event.data);
  };
  window.addEventListener('message', listener);
  return () => window.removeEventListener('message', listener);
}

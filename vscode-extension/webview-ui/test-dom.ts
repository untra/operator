import { GlobalRegistrator } from "@happy-dom/global-registrator";

if (globalThis.document === undefined) {
  GlobalRegistrator.register();
}

import { useState, type ReactNode } from "react";
import { Provider } from "react-redux";
import { createApiStore } from "./adapter";

export function ApiProvider({ children }: { children: ReactNode }) {
  const [store] = useState(createApiStore);
  return <Provider store={store}>{children}</Provider>;
}

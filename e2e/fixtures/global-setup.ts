import { BASE_URL } from "./env";

const READY_TIMEOUT_MS = 60_000;
const POLL_MS = 500;

async function healthy(): Promise<boolean> {
  try {
    return (await fetch(`${BASE_URL}/readyz`)).ok;
  } catch {
    return false;
  }
}

export default async function waitForServer(): Promise<void> {
  const deadline = Date.now() + READY_TIMEOUT_MS;
  while (!(await healthy())) {
    if (Date.now() > deadline) {
      throw new Error(`operator at ${BASE_URL} not healthy after ${READY_TIMEOUT_MS}ms`);
    }
    await new Promise((resolve) => setTimeout(resolve, POLL_MS));
  }
}

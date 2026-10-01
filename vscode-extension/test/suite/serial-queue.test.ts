import * as assert from "node:assert";
import { SerialQueue } from "../../src/serial-queue";

function deferred(): { promise: Promise<void>; resolve: () => void } {
  let resolve: () => void = () => undefined;
  const promise = new Promise<void>((r) => {
    resolve = r;
  });
  return { promise, resolve };
}

suite("SerialQueue", () => {
  test("runs tasks one at a time in submission order", async () => {
    const queue = new SerialQueue();
    const order: string[] = [];
    const gate = deferred();

    const first = queue.run(async () => {
      order.push("first:start");
      await gate.promise;
      order.push("first:end");
    });
    const second = queue.run(async () => {
      order.push("second");
      return Promise.resolve();
    });

    await Promise.resolve();
    assert.deepStrictEqual(order, ["first:start"]);
    gate.resolve();
    await Promise.all([first, second]);
    assert.deepStrictEqual(order, ["first:start", "first:end", "second"]);
  });

  test("a throwing task rejects its own promise without blocking later tasks", async () => {
    const queue = new SerialQueue();
    const failure = new Error("write failed");

    const failed = queue.run(() => Promise.reject(failure));
    const after = queue.run(() => Promise.resolve("ran"));

    await assert.rejects(failed, failure);
    assert.strictEqual(await after, "ran");
  });
});

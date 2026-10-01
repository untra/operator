/**
 * Runs async tasks one at a time in submission order. A task that throws
 * rejects only its own promise; later tasks still run.
 */
export class SerialQueue {
  private _tail: Promise<void> = Promise.resolve();

  run<T>(task: () => Promise<T>): Promise<T> {
    const result = this._tail.then(task);
    this._tail = result.then(
      () => undefined,
      () => undefined,
    );
    return result;
  }
}

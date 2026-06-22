import { describe, expect, test } from "bun:test";

import { startExclusivePolling, type PollingScheduler } from "./polling";

describe("startExclusivePolling", () => {
  test("runs immediately, prevents overlap and stops the interval", async () => {
    let intervalHandler: (() => void) | undefined;
    let clearedInterval: number | undefined;
    let resolveTask: (() => void) | undefined;
    let runCount = 0;
    const scheduler: PollingScheduler = {
      clearInterval: (intervalId) => {
        clearedInterval = intervalId;
      },
      setInterval: (handler) => {
        intervalHandler = handler;
        return 42;
      },
    };
    const task = (): Promise<void> => {
      runCount += 1;

      return new Promise((resolve) => {
        resolveTask = resolve;
      });
    };

    const stop = startExclusivePolling(task, 5_000, scheduler);

    expect(runCount).toBe(1);
    intervalHandler?.();
    expect(runCount).toBe(1);

    resolveTask?.();
    await Promise.resolve();
    intervalHandler?.();
    expect(runCount).toBe(2);

    stop();
    expect(clearedInterval).toBe(42);
  });
});

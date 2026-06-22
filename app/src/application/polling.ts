export type PollingScheduler = {
  clearInterval: (intervalId: number) => void;
  setInterval: (handler: () => void, intervalMs: number) => number;
};

export function startExclusivePolling(
  task: () => Promise<void>,
  intervalMs: number,
  scheduler: PollingScheduler = window,
): () => void {
  let inFlight = false;

  const run = async (): Promise<void> => {
    if (inFlight) {
      return;
    }

    inFlight = true;

    try {
      await task();
    } finally {
      inFlight = false;
    }
  };

  void run();
  const intervalId = scheduler.setInterval(() => {
    void run();
  }, intervalMs);

  return () => scheduler.clearInterval(intervalId);
}

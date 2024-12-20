interface RetryOptions {
  retries?: number;
  delay?: number;
}
const DEFAULT_RETRY_OPTIONS: RetryOptions = {
  retries: 3,
  delay: 0,
};
export async function retry<T>(func: () => Promise<T>, options: RetryOptions = {}): Promise<T> {
  const { retries, delay } = Object.assign({}, DEFAULT_RETRY_OPTIONS, options);
  try {
    return await func();
  } catch (error) {
    if (retries && retries > 0) {
      if (delay) {
        await new Promise((resolve) => setTimeout(resolve, delay));
      }
      return retry(func, { retries: retries - 1, delay });
    } else {
      throw error;
    }
  }
}

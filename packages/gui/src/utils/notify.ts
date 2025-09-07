import { toast } from "svelte-sonner";
import { error as logError } from "$utils/error";

interface NotifyOptions {
  description?: string;
}

/**
 * Success notification - logs to console and shows toast
 */
export function notifySuccess(message: string, options?: NotifyOptions) {
  console.info(message);
  toast.success(message, options);
}

/**
 * Error notification - logs to console with error util and shows toast
 */
export function notifyError(
  message: string,
  error?: unknown,
  options?: NotifyOptions,
) {
  const errorMessage = error ? `${message}: ${error}` : message;
  logError(errorMessage);

  toast.error(message, {
    description: options?.description || (error ? String(error) : undefined),
  });
}

/**
 * Info notification - logs to console and shows toast
 */
export function notifyInfo(message: string, options?: NotifyOptions) {
  console.info(message);
  toast.info(message, options);
}

/**
 * Warning notification - logs to console and shows toast
 */
export function notifyWarning(message: string, options?: NotifyOptions) {
  console.info(`Warning: ${message}`);
  toast.warning(message, options);
}

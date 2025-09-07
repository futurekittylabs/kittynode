import { toast } from "svelte-sonner";

export function error(message: string) {
  console.error(message);
  toast.error(message);
}

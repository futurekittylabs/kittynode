declare module "semver" {
  export function clean(version: string, options?: unknown): string | null;
  export function gt(
    version: string,
    other: string,
    options?: unknown,
  ): boolean;
}

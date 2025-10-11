import { redirect } from "@sveltejs/kit";
import cliRelease from "$lib/cli-release.json";

export function load() {
  const { version } = cliRelease as { version: string };
  redirect(
    307,
    `https://github.com/futurekittylabs/kittynode/releases/download/kittynode-cli-${version}/kittynode-cli-installer.ps1`,
  );
}

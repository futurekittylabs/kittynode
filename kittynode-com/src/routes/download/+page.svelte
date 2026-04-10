<script lang="ts">
import { AppWindowMac, Monitor, Terminal } from "@lucide/svelte";
import cliRelease from "$lib/cli-release.json";
import * as Code from "$lib/components/ui/code";
import * as Tabs from "$lib/components/ui/tabs/index.js";

const { version: cliVersion, date: cliPubDate } = cliRelease;

const baseUrl = "https://github.com/futurekittylabs/kittynode";
const changelogUrl = `${baseUrl}/releases`;
const cliInstallCommandUnix = `curl --proto '=https' --tlsv1.2 -LsSf https://kittynode.com/sh | sh`;
const cliInstallCommandHomebrew = "brew install futurekittylabs/tap/kittynode";
const cliInstallCommandWindows = `powershell -ExecutionPolicy Bypass -c "irm https://kittynode.com/ps1 | iex"`;

const releaseDateFormatter = new Intl.DateTimeFormat("en-US", {
  month: "long",
  day: "numeric",
  year: "numeric",
});

function formatReleaseDate(pubDate: string | undefined): string {
  if (!pubDate) {
    return "";
  }
  const parsed = new Date(pubDate);
  if (Number.isNaN(parsed.getTime())) {
    return "";
  }
  return releaseDateFormatter.format(parsed);
}

const cliReleaseDate = formatReleaseDate(cliPubDate);

const cliVersionLabel = `Version ${cliVersion}`;
const installsContainerClass =
  "mx-auto flex w-full flex-col gap-4 max-w-full sm:max-w-xl lg:max-w-[48rem]";
</script>

<div class="my-16">
  <div class="mb-8 text-center">
    <h1 class="text-2xl font-semibold mb-2">Install Kittynode</h1>
  </div>
  <div>
    <div class="mb-7 text-center">
      <p class="text-base font-medium">
        <a href={changelogUrl} class="link">{cliVersionLabel}</a>
        {#if cliReleaseDate}
          — {cliReleaseDate}
        {/if}
      </p>
    </div>
    <div class={installsContainerClass}>
      <div class="rounded-lg border bg-card p-5 space-y-3">
        <div class="flex items-center gap-3">
          <div class="rounded-md bg-muted p-1.5">
            <Terminal class="h-5 w-5" />
          </div>
          <div>
            <h3 class="text-base font-medium">Linux</h3>
            <p class="text-sm text-muted-foreground">
              Open a terminal and enter the following:
            </p>
          </div>
        </div>
        <Code.Root
          lang="bash"
          class="w-full"
          code={cliInstallCommandUnix}
          hideLines
        >
          <Code.CopyButton variant="secondary" />
        </Code.Root>
      </div>
      <div class="rounded-lg border bg-card p-5 space-y-3">
        <div class="flex items-center gap-3">
          <div class="rounded-md bg-muted p-1.5">
            <AppWindowMac class="h-5 w-5" />
          </div>
          <div>
            <h3 class="text-base font-medium">macOS</h3>
            <p class="text-sm text-muted-foreground">
              Choose your preferred install method:
            </p>
          </div>
        </div>
        <Tabs.Root value="shell">
          <Tabs.List
            aria-label="macOS install options"
            class="mb-3 flex gap-2"
          >
            <Tabs.Trigger value="shell" class="px-3">shell</Tabs.Trigger>
            <Tabs.Trigger value="homebrew" class="px-3">
              homebrew
            </Tabs.Trigger>
          </Tabs.List>
          <Tabs.Content value="shell">
            <Code.Root
              lang="bash"
              class="w-full"
              code={cliInstallCommandUnix}
              hideLines
            >
              <Code.CopyButton variant="secondary" />
            </Code.Root>
          </Tabs.Content>
          <Tabs.Content value="homebrew">
            <Code.Root
              lang="bash"
              class="w-full"
              code={cliInstallCommandHomebrew}
              hideLines
            >
              <Code.CopyButton variant="secondary" />
            </Code.Root>
          </Tabs.Content>
        </Tabs.Root>
      </div>
      <div class="rounded-lg border bg-card p-5 space-y-3">
        <div class="flex items-center gap-3">
          <div class="rounded-md bg-muted p-1.5">
            <Monitor class="h-5 w-5" />
          </div>
          <div>
            <h3 class="text-base font-medium">Windows</h3>
            <p class="text-sm text-muted-foreground">
              Open PowerShell and enter the following:
            </p>
          </div>
        </div>
        <Code.Root
          lang="bash"
          class="w-full"
          code={cliInstallCommandWindows}
          hideLines
        >
          <Code.CopyButton variant="secondary" />
        </Code.Root>
      </div>
    </div>
  </div>
</div>

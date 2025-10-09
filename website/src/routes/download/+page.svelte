<script lang="ts">
import {
  Terminal,
  Monitor,
  Download,
  AppWindowMac,
  CircleQuestionMark,
  ChevronDown,
  Package,
} from "@lucide/svelte";
import { Button } from "$lib/components/ui/button/index.js";
import {
  Collapsible,
  CollapsibleTrigger,
  CollapsibleContent,
} from "$lib/components/ui/collapsible/index.js";
import { CopyButton } from "$lib/components/ui/copy-button/index.js";
import appRelease from "$lib/app-release.json";
import cliRelease from "$lib/cli-release.json";

let linuxHelpOpen = false;

const { version: appVersion, date: releaseDate } = appRelease;
const { version: cliVersion } = cliRelease;

const baseUrl = "https://github.com/futurekittylabs/kittynode";
const changelogUrl = `${baseUrl}/releases`;
const releaseUrl = `${baseUrl}/releases/download/kittynode-app-${appVersion}`;
const discordUrl = "https://discord.kittynode.com";
const cliInstallCommand = `curl --proto '=https' --tlsv1.2 -LsSf https://github.com/futurekittylabs/kittynode/releases/download/kittynode-cli-${cliVersion}/kittynode-cli-installer.sh | sh`;

const downloads = [
  {
    name: "Linux",
    icon: Package,
    requirements: "Linux (x86_64)",
    options: [
      {
        label: ".deb",
        url: `${releaseUrl}/Kittynode_${appVersion}_amd64.deb`,
      },
      {
        label: ".rpm",
        url: `${releaseUrl}/Kittynode-${appVersion}-1.x86_64.rpm`,
      },
    ],
    layout: "stacked",
  },
  {
    name: "macOS",
    icon: AppWindowMac,
    requirements: "macOS 10.15+ (Apple Silicon)",
    primary: {
      label: ".dmg",
      url: `${releaseUrl}/Kittynode_${appVersion}_aarch64.dmg`,
    },
    options: [
      {
        label: ".app.tar.gz",
        url: `${releaseUrl}/Kittynode_aarch64.app.tar.gz`,
      },
    ],
  },
  {
    name: "Windows",
    icon: Monitor,
    requirements: "Windows 7+ (x86_64)",
    primary: {
      label: ".exe",
      url: `${releaseUrl}/Kittynode_${appVersion}_x64-setup.exe`,
    },
    options: [
      {
        label: ".msi",
        url: `${releaseUrl}/Kittynode_${appVersion}_x64_en-US.msi`,
      },
    ],
  },
];
</script>

<div class="py-16">
  <!-- Header -->
  <div class="mb-8 text-center">
    <h1 class="text-3xl font-medium mb-2">Download Kittynode</h1>
    <p class="text-sm text-muted-foreground mb-4">
      Version {appVersion} • {releaseDate}
    </p>
    <a href={changelogUrl} class="link text-sm"> View changelog</a>
  </div>
  <!-- Download cards -->
  <div class="grid gap-4 min-[900px]:grid-cols-3">
    {#each downloads as info}
      <div class="rounded-lg border bg-card p-5">
        <div class="flex items-center gap-3 mb-4">
          <div class="p-1.5 rounded-md bg-muted">
            <svelte:component this={info.icon} class="h-5 w-5" />
          </div>
          <div>
            <h2 class="text-base font-medium">{info.name}</h2>
            <p class="text-xs text-muted-foreground">{info.requirements}</p>
          </div>
        </div>

        <div class="space-y-2">
          {#if info.primary}
            <Button
              href={info.primary.url}
              size="sm"
              class="w-full gap-2"
              variant="default"
            >
              <Download class="h-3.5 w-3.5" />
              {info.primary.label}
            </Button>
          {/if}

          {#if info.options && info.options.length > 0}
            {#if info.layout === "stacked"}
              {#each info.options as option}
                <Button
                  href={option.url}
                  size="sm"
                  class="w-full gap-2"
                  variant="outline"
                >
                  <Download class="h-3.5 w-3.5" />
                  {option.label}
                </Button>
              {/each}
            {:else}
              <div class="flex gap-2">
                {#each info.options as option}
                  <Button
                    href={option.url}
                    size="sm"
                    variant="outline"
                    class="flex-1 gap-2"
                  >
                    <Download class="h-3.5 w-3.5" />
                    {option.label}
                  </Button>
                {/each}
              </div>
            {/if}
          {/if}
        </div>
      </div>
    {/each}
  </div>

  <div class="mt-10 overflow-hidden rounded-lg border bg-card">
    <div class="flex flex-col gap-6 px-6 py-6">
      <div class="flex items-center gap-3">
        <div class="rounded-md bg-muted p-2">
          <Terminal class="h-5 w-5" />
        </div>
        <h2 class="text-lg font-semibold">Kittynode CLI</h2>
      </div>
      <p class="text-sm text-muted-foreground">
        Manage your node infrastructure directly from the terminal.
      </p>
      <div>
        <p
          class="text-sm font-semibold text-muted-foreground"
        >
          Run the following in your terminal:
        </p>
        <div class="mt-3 rounded-lg border bg-background/80 px-4 py-3 font-mono text-sm">
          <div
            class="flex flex-nowrap items-start gap-3 min-[640px]:items-center min-[640px]:justify-between"
          >
            <div class="min-w-0 overflow-x-auto px-2 pt-1 pb-2">
              <code class="block whitespace-nowrap leading-snug pr-8">
                {cliInstallCommand}
              </code>
            </div>
            <CopyButton
              class="shrink-0 min-[640px]:self-center"
              aria-label="Copy install command"
              text={cliInstallCommand}
            />
          </div>
        </div>
        <!-- No additional prerequisites for installer script -->
      </div>
    </div>
  </div>

  <Collapsible bind:open={linuxHelpOpen} class="mt-10 max-w-2xl mx-auto">
    <div
      class="rounded-lg has-[:focus-visible]:ring-2 has-[:focus-visible]:ring-ring has-[:focus-visible]:ring-offset-2 has-[:focus-visible]:ring-offset-background"
    >
      <div class="overflow-hidden rounded-lg border">
        <CollapsibleTrigger
          class="flex w-full items-center justify-between gap-3 px-4 py-3 text-left text-sm font-medium transition-colors hover:bg-muted/60 focus-visible:outline-none"
        >
          <span class="flex items-center gap-2">
            <CircleQuestionMark class="h-4 w-4 text-link" />
            Looking for another Linux package format?
          </span>
          <ChevronDown
            class={`h-4 w-4 transition-transform ${linuxHelpOpen ? "rotate-180" : ""}`}
          />
        </CollapsibleTrigger>
        <CollapsibleContent
          class="space-y-3 px-4 pb-4 pt-1 text-sm text-muted-foreground"
        >
          <p>
            We're expanding our Linux packaging support beyond the options
            listed above.
          </p>
          <p>
            Please reach out on <a href={discordUrl} class="link">Discord</a> or
            <a href={baseUrl} class="link">GitHub</a> if your distro is not supported
            — we want to support your system and will prioritize it!
          </p>
        </CollapsibleContent>
      </div>
    </div>
  </Collapsible>
</div>

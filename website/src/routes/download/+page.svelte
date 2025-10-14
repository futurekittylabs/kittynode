<script lang="ts">
import { Monitor, Download, AppWindowMac, Package } from "@lucide/svelte";
import { Button } from "$lib/components/ui/button/index.js";
import * as Code from "$lib/components/ui/code";
import * as Tabs from "$lib/components/ui/tabs/index.js";
import appRelease from "$lib/app-release.json";
import cliRelease from "$lib/cli-release.json";

const { version: appVersion } = appRelease;
const { version: cliVersion } = cliRelease;

const baseUrl = "https://github.com/futurekittylabs/kittynode";
const changelogUrl = `${baseUrl}/releases`;
const releaseUrl = `${baseUrl}/releases/download/kittynode-app-${appVersion}`;
const cliInstallCommandUnix = `curl --proto '=https' --tlsv1.2 -LsSf https://kittynode.com/sh | sh`;
const cliInstallCommandWindows = `powershell -ExecutionPolicy Bypass -c "irm https://kittynode.com/ps1 | iex"`;

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

<div class="mt-16">
  <div class="mb-4 text-center">
    <h1 class="text-2xl font-semibold mb-2">Download Kittynode App</h1>
    <p class="text-sm text-muted-foreground">
      <a href={changelogUrl} class="link">Version {appVersion}</a>
    </p>
  </div>
  <p class="mx-auto mb-6 max-w-2xl text-center text-sm text-muted-foreground">
    A desktop app for securely operating Ethereum.
  </p>
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

  <div class="mt-16">
    <div class="mb-4 text-center">
      <h1 class="text-2xl font-semibold mb-2">Install Kittynode CLI</h1>
      <p class="text-sm text-muted-foreground">
        <a href={changelogUrl} class="link">Version {cliVersion}</a>
      </p>
    </div>
    <p class="mx-auto mb-6 max-w-2xl text-center text-sm text-muted-foreground">
      A CLI app for securely operating Ethereum.
    </p>
    <Tabs.Root value="linux/macos">
      <Tabs.List>
        <Tabs.Trigger value="linux/macos">Linux / macOS</Tabs.Trigger>
        <Tabs.Trigger value="windows">Windows</Tabs.Trigger>
      </Tabs.List>
      <Tabs.Content value="linux/macos">
        <Code.Root lang="bash" class="w-full" code={cliInstallCommandUnix} hideLines>
          <Code.CopyButton />
        </Code.Root>
      </Tabs.Content>
      <Tabs.Content value="windows">
        <Code.Root lang="bash" class="w-full" code={cliInstallCommandWindows} hideLines>
          <Code.CopyButton />
        </Code.Root>
      </Tabs.Content>
    </Tabs.Root>
  </div>
</div>

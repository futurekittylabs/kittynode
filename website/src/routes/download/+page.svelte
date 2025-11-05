<script lang="ts">
import { Monitor, Download, AppWindowMac, Terminal } from "@lucide/svelte";
import { Button } from "$lib/components/ui/button/index.js";
import * as Code from "$lib/components/ui/code";
import * as Tabs from "$lib/components/ui/tabs/index.js";
import appRelease from "$lib/app-release.json";
import cliRelease from "$lib/cli-release.json";

const { version: appVersion, pub_date: appPubDate } = appRelease;
const { version: cliVersion, date: cliPubDate } = cliRelease;

const baseUrl = "https://github.com/futurekittylabs/kittynode";
const changelogUrl = `${baseUrl}/releases`;
const releaseUrl = `${baseUrl}/releases/download/kittynode-app-${appVersion}`;
const cliInstallCommandUnix = `curl --proto '=https' --tlsv1.2 -LsSf https://kittynode.com/sh | sh`;
const cliInstallCommandWindows = `powershell -ExecutionPolicy Bypass -c "irm https://kittynode.com/ps1 | iex"`;

const releaseDateFormatter = new Intl.DateTimeFormat("en-US", {
  month: "long",
  day: "numeric",
  year: "numeric",
});

function formatReleaseDate(pubDate: string | undefined): string {
  if (!pubDate) return "";
  const parsed = new Date(pubDate);
  if (Number.isNaN(parsed.getTime())) {
    return "";
  }
  return releaseDateFormatter.format(parsed);
}

const appReleaseDate = formatReleaseDate(appPubDate);
const cliReleaseDate = formatReleaseDate(cliPubDate);

const appVersionLabel = `Version ${appVersion}`;
const cliVersionLabel = `Version ${cliVersion}`;

const downloads = [
  {
    name: "Linux",
    icon: Terminal,
    requirements: "Linux",
    variants: [
      {
        label: "x86_64",
        downloads: [
          {
            label: ".AppImage",
            url: `${releaseUrl}/Kittynode_${appVersion}_amd64_linux.AppImage`,
            span: "full",
          },
          {
            label: ".deb",
            url: `${releaseUrl}/Kittynode_${appVersion}_amd64_linux.deb`,
          },
          {
            label: ".rpm",
            url: `${releaseUrl}/Kittynode-${appVersion}-1.x86_64_linux.rpm`,
          },
        ],
      },
    ],
  },
  {
    name: "macOS",
    icon: AppWindowMac,
    requirements: "macOS 10.15+",
    variants: [
      {
        label: "Apple Silicon",
        downloads: [
          {
            label: ".dmg",
            url: `${releaseUrl}/Kittynode_${appVersion}_aarch64_darwin.dmg`,
          },
          {
            label: ".app.tar.gz",
            url: `${releaseUrl}/Kittynode_darwin_aarch64.app.tar.gz`,
          },
        ],
      },
      {
        label: "x86_64",
        downloads: [
          {
            label: ".dmg",
            url: `${releaseUrl}/Kittynode_${appVersion}_x64_darwin.dmg`,
          },
          {
            label: ".app.tar.gz",
            url: `${releaseUrl}/Kittynode_darwin_x64.app.tar.gz`,
          },
        ],
      },
    ],
  },
  {
    name: "Windows",
    icon: Monitor,
    requirements: "Windows 7+",
    variants: [
      {
        label: "x86_64",
        downloads: [
          {
            label: ".exe",
            url: `${releaseUrl}/Kittynode_${appVersion}_x64-setup_windows.exe`,
          },
          {
            label: ".msi",
            url: `${releaseUrl}/Kittynode_${appVersion}_x64_en-US_windows.msi`,
          },
        ],
      },
    ],
  },
];
</script>

<div class="my-16">
  <div class="mb-8 text-center">
    <h1 class="text-2xl font-semibold mb-2">Install Kittynode</h1>
    <p class="mx-auto max-w-2xl text-sm text-muted-foreground">
      We recommend the desktop app for most users! The CLI app is great for
      users that want to run a node remotely (e.g., on a dedicated Linux server)
      or prefer the simplicity of a CLI.
    </p>
  </div>

  <Tabs.Root value="desktop">
    <Tabs.List
      class="mx-auto mb-4 flex w-full max-w-md flex-wrap justify-center gap-2"
    >
      <Tabs.Trigger value="desktop" class="px-4">Desktop</Tabs.Trigger>
      <Tabs.Trigger value="cli" class="px-4">Command line</Tabs.Trigger>
    </Tabs.List>

    <Tabs.Content value="desktop" class="space-y-8">
      <div>
        <div class="mb-7 text-center">
          <p class="text-base font-medium">
            <a href={changelogUrl} class="link">{appVersionLabel}</a>
            {#if appReleaseDate}
              — {appReleaseDate}
            {/if}
          </p>
        </div>
        <div
          class="mx-auto flex w-full flex-col gap-4 max-w-full sm:max-w-xl lg:max-w-2xl"
        >
          {#each downloads as info}
            <div class="rounded-lg border bg-card p-5">
              <div
                class="flex flex-col gap-4 md:flex-row md:items-start md:justify-between"
              >
                <div class="flex items-center gap-3">
                  <div class="rounded-md bg-muted p-1.5">
                    <svelte:component this={info.icon} class="h-5 w-5" />
                  </div>
                  <div>
                    <h3 class="text-base font-medium">{info.name}</h3>
                    <p class="text-sm text-muted-foreground">
                      {info.requirements}
                    </p>
                  </div>
                </div>
                <div class="flex-1 space-y-4 w-full md:w-auto md:max-w-[18rem]">
                  {#each info.variants as variant}
                    <div class="space-y-2">
                      <p class="text-sm font-medium text-muted-foreground">
                        {variant.label}
                      </p>
                      {#if variant.downloads.some((download) => download.span === "full")}
                        {#each variant.downloads as download}
                          {#if download.span === "full"}
                            <Button
                              href={download.url}
                              size="sm"
                              class="w-full justify-center gap-2"
                              variant={variant.downloads.indexOf(download) === 0
                                ? "default"
                                : "outline"}
                            >
                              <Download class="h-3.5 w-3.5" />
                              {download.label}
                            </Button>
                          {/if}
                        {/each}
                      {/if}
                      {#if variant.downloads.some((download) => download.span !== "full")}
                        <div class="grid gap-2 min-[420px]:grid-cols-2">
                          {#each variant.downloads as download}
                            {#if download.span !== "full"}
                              <Button
                                href={download.url}
                                size="sm"
                                class="w-full justify-center gap-2"
                                variant={variant.downloads.indexOf(download) ===
                                0
                                  ? "default"
                                  : "outline"}
                              >
                                <Download class="h-3.5 w-3.5" />
                                {download.label}
                              </Button>
                            {/if}
                          {/each}
                        </div>
                      {/if}
                    </div>
                  {/each}
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>
    </Tabs.Content>

    <Tabs.Content value="cli" class="space-y-8">
      <div>
        <div class="mb-7 text-center">
          <p class="text-base font-medium">
            <a href={changelogUrl} class="link">{cliVersionLabel}</a>
            {#if cliReleaseDate}
              — {cliReleaseDate}
            {/if}
          </p>
        </div>
        <div class="mx-auto flex w-full flex-col gap-4">
          <div class="rounded-lg border bg-card p-5 space-y-3">
            <div class="flex items-center gap-3">
              <div class="rounded-md bg-muted p-1.5">
                <Terminal class="h-5 w-5" />
              </div>
              <div>
                <h3 class="text-base font-medium">Linux / macOS</h3>
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
    </Tabs.Content>
  </Tabs.Root>
</div>

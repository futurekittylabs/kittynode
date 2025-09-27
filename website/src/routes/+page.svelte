<script lang="ts">
import { onMount } from "svelte";
import { ArrowUpRight, Download } from "@lucide/svelte";
import { Button } from "$lib/components/ui/button/index.js";
import releaseInfo from "$lib/release.json";

type KnownOS = "linux" | "mac" | "windows" | "ios" | "android" | "unknown";

const { version } = releaseInfo;
const repoUrl = "https://github.com/futurekittylabs/kittynode";
const releaseBaseUrl = `${repoUrl}/releases/download/kittynode-app@${version}`;

const downloads = {
  linux: `${releaseBaseUrl}/Kittynode_${version}_amd64.deb`,
  mac: `${releaseBaseUrl}/Kittynode_${version}_aarch64.dmg`,
  windows: `${releaseBaseUrl}/Kittynode_${version}_x64-setup.exe`,
} as const;

let downloadHref = "/download";
let downloadButtonText = "Download now";
let showFallback = true;

function setButtonState(os: KnownOS) {
  if (os === "linux") {
    downloadHref = downloads.linux;
    downloadButtonText = "Download .deb for Linux";
    showFallback = false;
    return;
  }

  if (os === "mac") {
    downloadHref = downloads.mac;
    downloadButtonText = "Download for macOS";
    showFallback = false;
    return;
  }

  if (os === "windows") {
    downloadHref = downloads.windows;
    downloadButtonText = "Download for Windows";
    showFallback = false;
    return;
  }
}

function detectOS(): KnownOS {
  if (typeof navigator === "undefined") {
    return "unknown";
  }

  const userAgent = navigator.userAgent.toLowerCase();

  // Check mobile signatures first; their UAs also contain desktop keywords.
  if (
    userAgent.includes("iphone") ||
    userAgent.includes("ipad") ||
    userAgent.includes("ipod")
  ) {
    return "ios";
  }

  if (userAgent.includes("android")) {
    return "android";
  }

  // Ordering from most to least specific.
  if (userAgent.includes("windows")) {
    return "windows";
  }

  if (userAgent.includes("mac") || userAgent.includes("macintosh")) {
    return "mac";
  }

  if (userAgent.includes("linux") || userAgent.includes("x11")) {
    return "linux";
  }

  return "unknown";
}

onMount(() => {
  setButtonState(detectOS());
});
</script>

<div class="flex flex-1 items-center justify-center">
  <div class="container max-w-6xl mx-auto px-6 py-16">
    <div class="mx-auto max-w-3xl text-center">
      <h1 class="hero-heading">
        Operate the world<br />computer
      </h1>
      <p class="hero-subtitle mt-6 text-muted-foreground">
        Kittynode is a control center for world computer operators.
      </p>
      <img src="/black-kitty.gif" alt="Black Kitty" class="nyan-cat mt-12" />
      <div class="mt-8 flex flex-col items-center gap-4">
        <Button href={downloadHref} size="lg" class="gap-2">
          <Download class="h-5 w-5" />
          {downloadButtonText}
        </Button>
        {#if showFallback}
          <p class="text-sm text-muted-foreground">
            Available for Linux, macOS, and Windows.
          </p>
        {:else}
          <p class="text-sm text-muted-foreground">
            Need something else? <a href="/download" class="link">See all downloads</a>.
          </p>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .hero-heading {
    font-size: 2.25rem;
    font-weight: 500;
  }

  .hero-subtitle {
    font-size: 1rem;
  }

  @media (min-width: 640px) {
    .hero-heading {
      font-size: 3.5rem;
      font-weight: 500;
    }

    .hero-subtitle {
      font-size: 1.125rem;
    }
  }

  .nyan-cat {
    width: 180px;
    height: auto;
    min-width: 180px;
    max-width: none;
    image-rendering: pixelated;
    display: block;
    margin-left: auto;
    margin-right: auto;
  }

  @media (min-width: 640px) {
    .nyan-cat {
      width: 220px;
      min-width: 220px;
    }
  }
</style>

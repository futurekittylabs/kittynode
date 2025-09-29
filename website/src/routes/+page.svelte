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

<div class="hero">
  <div class="w-full">
    <div class="mx-auto text-center">
      <h1
        class="text-balance font-medium leading-snug tracking-tight text-[clamp(2.25rem,3vw+0.75rem,3.25rem)]"
      >
        Run the world computer
      </h1>
      <div class="mx-auto max-w-[68ch] mt-[clamp(1.25rem,3vh,2rem)]">
        <div class="flex flex-col items-center space-y-[clamp(1.25rem,3.5vh,2.25rem)]">
          <p
          class="text-[clamp(1.15rem,1.1vw+0.35rem,1.3rem)] text-muted-foreground"
        >
            Kittynode is a control center for world computer operators.
          </p>
          <img
            src="/black-kitty.gif"
            alt="Black Kitty"
            class="nyan-cat mx-auto w-[clamp(160px,20vw,210px)]"
          />
          <div class="flex flex-col items-center gap-[clamp(0.9rem,2.2vh,1.5rem)]">
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
  </div>
</div>

<style>
  .nyan-cat {
    image-rendering: pixelated;
  }

  .text-balance {
    text-wrap: balance;
  }

  .hero {
    /* Prefer safe viewport height when supported (set below) */
    --vh: 1vh;

    display: grid;
    place-content: center;
    margin-block-end: 0;

    /* Responsive frame for breathing room around centered content */
    min-block-size: calc(70 * var(--vh));
    padding-block: clamp(1rem, calc(4 * var(--vh)), 2.5rem);
  }
  @supports (height: 100svh) {
    .hero {
      --vh: 1svh;
    }
  }
</style>

<script lang="ts">
import { ArrowRight } from "@lucide/svelte";
import { Button } from "$lib/components/ui/button/index.js";
import * as Tabs from "$lib/components/ui/tabs/index.js";

const frontends = ["app", "cli", "mobile"] as const;
type FrontendId = (typeof frontends)[number];

const screenshots = {
  app: {
    label: "Desktop",
    alt: "Kittynode desktop app dashboard",
    height: 602,
  },
  cli: {
    label: "Command line",
    alt: "Kittynode command-line interface overview",
    height: 602,
  },
} as const;

const labels: Record<FrontendId, string> = {
  app: screenshots.app.label,
  cli: screenshots.cli.label,
  mobile: "Mobile",
};

const src = (id: keyof typeof screenshots, theme: "light" | "dark") =>
  `/images/kittynode-${id}-${theme}-960.webp`;
const srcset = (id: keyof typeof screenshots, theme: "light" | "dark") =>
  `${src(id, theme)} 960w, ${src(id, theme).replace("-960", "-1920")} 1920w`;
const sizes = "(min-width: 1024px) 46vw, 96vw";
const width = 960;
const fallbackHeight = Math.max(
  ...Object.values(screenshots).map(({ height }) => height),
);
const isScreenshot = (id: FrontendId): id is keyof typeof screenshots =>
  id !== "mobile";

let active: FrontendId = "app";
let screenshotId: keyof typeof screenshots | null = "app";
let screenshot: (typeof screenshots)[keyof typeof screenshots] | null =
  screenshots.app;
let aspectRatio: string = `${width} / ${fallbackHeight}`;

$: {
  screenshotId = isScreenshot(active) ? active : null;
  screenshot = screenshotId ? screenshots[screenshotId] : null;
  aspectRatio = `${width} / ${screenshot?.height ?? fallbackHeight}`;
}
</script>

<div class="hero">
  <div class="grid w-full gap-10 text-left lg:grid-cols-[minmax(0,0.9fr)_minmax(0,1.1fr)]">
    <div class="flex flex-col gap-6">
      <h1 class="text-balance font-medium leading-tight tracking-tight text-[clamp(2.25rem,2.6vw+1rem,3.25rem)]">Run the world computer</h1>
      <p class="text-[clamp(1.05rem,1vw+0.3rem,1.25rem)] text-muted-foreground">Kittynode is a control center for world computer operators.</p>
      <div class="flex flex-col items-start gap-4">
        <Button href="/download" size="lg" class="gap-2">
          Get started
          <ArrowRight class="h-5 w-5" />
        </Button>
        <p class="text-sm text-muted-foreground">Available for Linux, macOS, and Windows.</p>
      </div>
    </div>

    <div class="flex flex-col gap-5">
      <div class="self-start">
        <Tabs.Root bind:value={active}>
          <Tabs.List aria-label="Kittynode frontends" class="flex flex-wrap gap-2">
            {#each frontends as frontend}
              <Tabs.Trigger value={frontend} class="px-4">{labels[frontend]}</Tabs.Trigger>
            {/each}
          </Tabs.List>
        </Tabs.Root>
      </div>

      <div
        class="relative w-full overflow-hidden rounded-sm border border-border bg-muted/40"
        aria-live="polite"
        style:aspect-ratio={aspectRatio}
      >
        {#if screenshot && screenshotId}
          <picture class="h-full w-full">
            <source media="(prefers-color-scheme: dark)" srcset={srcset(screenshotId, "dark")} sizes={sizes} />
            <source media="(prefers-color-scheme: light)" srcset={srcset(screenshotId, "light")} sizes={sizes} />
            <img src={src(screenshotId, "light")} alt={screenshot.alt} width={width} height={screenshot.height} loading="eager" decoding="async" class="h-full w-full object-cover" />
          </picture>
        {:else}
          <div class="flex h-full flex-col items-center justify-center gap-4 p-6 text-center sm:gap-6 sm:p-8">
            <img
              src="/black-kitty.gif"
              alt="Animated kitty"
              width="160"
              height="120"
              class="nyan-cat w-[clamp(96px,34vw,160px)]"
            />
            <p class="text-[clamp(1rem,3vw,1.2rem)]">
              Join the <a href="https://discord.kittynode.com" class="link">Kittynode Discord</a> and send a message to get access!
            </p>
          </div>
        {/if}
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
    display: grid;
    place-content: center;
    margin-block-end: 0;
    min-block-size: min(100%, 100vh);
    padding-block: clamp(1.5rem, 6vh, 2.75rem);
  }

  @supports (height: 100dvh) {
    .hero {
      min-block-size: min(100%, 100dvh);
      padding-block: clamp(1.5rem, 6dvh, 3rem);
    }
  }

  @supports (height: 100svh) {
    .hero {
      min-block-size: min(100%, 100svh);
    }
  }

  :global(main:has(> .hero)) {
    display: flex;
    flex-direction: column;
    justify-content: center;
  }
</style>

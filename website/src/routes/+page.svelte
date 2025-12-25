<script lang="ts">
import { ArrowRight, Package, Layers, Wifi, Lock } from "@lucide/svelte";
import { Button } from "$lib/components/ui/button/index.js";
import * as Tabs from "$lib/components/ui/tabs/index.js";

const screenshots = {
  app: {
    label: "Desktop",
    alt: "Kittynode desktop app dashboard",
    height: 602,
  },
  cli: {
    label: "Terminal",
    alt: "Kittynode command-line interface overview",
    height: 602,
  },
} as const;

type ScreenshotId = keyof typeof screenshots;
const screenshotIds = Object.keys(screenshots) as ScreenshotId[];

const src = (id: ScreenshotId, theme: "light" | "dark") =>
  `/images/kittynode-${id}-${theme}-960.webp`;
const srcset = (id: ScreenshotId, theme: "light" | "dark") =>
  `${src(id, theme)} 960w, ${src(id, theme).replace("-960", "-1920")} 1920w`;

let active: ScreenshotId = $state("app");
</script>

<div class="page">
  <!-- Hero -->
  <section class="hero">
    <h1>Run Ethereum at home</h1>
    <p class="subtitle">
      Kittynode is a control center for world computer operators.
    </p>
    <div class="cta-group">
      <Button href="/download" size="lg" class="gap-2">
        Download
        <ArrowRight class="h-5 w-5" />
      </Button>
      <span class="platforms">Linux, macOS, Windows</span>
    </div>
  </section>

  <!-- Screenshot showcase -->
  <section class="showcase">
    <Tabs.Root bind:value={active}>
      <Tabs.List aria-label="Screenshot view" class="h-11 p-1">
        {#each screenshotIds as id}
          <Tabs.Trigger value={id} class="px-5 text-base">{screenshots[id].label}</Tabs.Trigger>
        {/each}
      </Tabs.List>
    </Tabs.Root>

    <div class="screenshot-wrapper">
      <picture>
        <source
          media="(prefers-color-scheme: dark)"
          srcset={srcset(active, "dark")}
          sizes="(min-width: 1200px) 1100px, 92vw"
        />
        <source
          media="(prefers-color-scheme: light)"
          srcset={srcset(active, "light")}
          sizes="(min-width: 1200px) 1100px, 92vw"
        />
        <img
          src={src(active, "light")}
          alt={screenshots[active].alt}
          width="1920"
          height={screenshots[active].height * 2}
          loading="eager"
          decoding="async"
        />
      </picture>
    </div>
  </section>

  <!-- Features -->
  <section class="features">
    <div class="feature">
      <div class="feature-icon">
        <Package class="h-6 w-6" />
      </div>
      <h2>Package ecosystem</h2>
      <p>
        Ethereum nodes install as packages. Each install is secure, consistent
        across systems, and atomic.
      </p>
    </div>

    <div class="feature">
      <div class="feature-icon">
        <Layers class="h-6 w-6" />
      </div>
      <h2>One core, every surface</h2>
      <p>
        CLI, desktop, and mobile apps share the same Rust core. Write once,
        run everywhere.
      </p>
    </div>

    <div class="feature">
      <div class="feature-icon">
        <Wifi class="h-6 w-6" />
      </div>
      <h2>Remote access</h2>
      <p>
        Monitor and manage your node from anywhere. Upgrade from your phone
        when it matters.
      </p>
    </div>

    <div class="feature">
      <div class="feature-icon">
        <Lock class="h-6 w-6" />
      </div>
      <h2>Minimal by default</h2>
      <p>
        Tiny, auditable codebase. Capabilities are opt-in. Read-only mode
        until you need more.
      </p>
    </div>
  </section>

  <!-- Bottom CTA -->
  <section class="bottom-cta">
    <a href="https://docs.kittynode.com" class="docs-link">
      Read the docs
      <ArrowRight class="h-4 w-4" />
    </a>
  </section>
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: clamp(4rem, 10vw, 6rem);
    padding-block: clamp(3rem, 8vw, 5rem);
  }

  /* Hero */
  .hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 1.25rem;
  }

  .hero h1 {
    font-size: clamp(2.25rem, 5.5vw, 3.5rem);
    font-weight: 500;
    letter-spacing: -0.02em;
    line-height: 1.1;
    text-wrap: balance;
  }

  .subtitle {
    font-size: clamp(1.1rem, 2vw, 1.35rem);
    color: var(--muted-foreground);
    line-height: 1.6;
  }

  .cta-group {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.75rem;
    margin-top: 0.25rem;
  }

  .platforms {
    font-family: "Space Grotesk Variable", sans-serif;
    font-size: 0.875rem;
    color: var(--muted-foreground);
  }

  /* Screenshot showcase - break out of container */
  .showcase {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1.25rem;
    width: 100vw;
    margin-inline: calc(50% - 50vw);
    padding-inline: max(1.5rem, calc(50vw - 600px));
  }

  .screenshot-wrapper {
    width: 100%;
    max-width: 1080px;
    border-radius: var(--radius-lg);
    overflow: hidden;
    border: 1px solid var(--border);
    background: var(--muted);
    box-shadow:
      0 6px 24px rgb(0 0 0 / 0.07),
      0 0 0 1px rgb(0 0 0 / 0.02);
  }

  .screenshot-wrapper img {
    width: 100%;
    height: auto;
    display: block;
  }

  /* Features */
  .features {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 2rem 2.5rem;
    max-width: 800px;
    margin-inline: auto;
    padding-inline: 1rem;
  }

  @media (max-width: 600px) {
    .features {
      grid-template-columns: 1fr;
    }
  }

  .feature {
    display: flex;
    flex-direction: column;
    gap: 0.625rem;
  }

  .feature-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2.75rem;
    height: 2.75rem;
    border-radius: var(--radius);
    background: var(--muted);
    color: var(--primary);
  }

  .feature h2 {
    font-family: "Space Grotesk Variable", sans-serif;
    font-size: 1.125rem;
    font-weight: 600;
    letter-spacing: -0.01em;
  }

  .feature p {
    font-size: 1rem;
    color: var(--muted-foreground);
    line-height: 1.6;
  }

  /* Bottom CTA */
  .bottom-cta {
    display: flex;
    justify-content: center;
  }

  .docs-link {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    font-family: "Space Grotesk Variable", sans-serif;
    font-size: 1rem;
    font-weight: 500;
    color: var(--muted-foreground);
    text-decoration: none;
    transition: color 0.15s ease;
  }

  .docs-link:hover {
    color: var(--foreground);
  }
</style>

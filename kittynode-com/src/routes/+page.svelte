<script lang="ts">
import { ArrowRight, Layers, Lock, Package, Wifi } from "@lucide/svelte";
import { Button } from "$lib/components/ui/button/index.js";

const screenshot = {
  alt: "Kittynode command-line interface overview",
  height: 602,
};

const src = (theme: "light" | "dark") =>
  `/images/kittynode-cli-${theme}-960.webp`;
const srcset = (theme: "light" | "dark") =>
  `${src(theme)} 960w, ${src(theme).replace("-960", "-1920")} 1920w`;
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
    <div class="screenshot-wrapper">
      <picture>
        <source
          media="(prefers-color-scheme: dark)"
          srcset={srcset("dark")}
          sizes="(min-width: 1200px) 1100px, 92vw"
        >
        <source
          media="(prefers-color-scheme: light)"
          srcset={srcset("light")}
          sizes="(min-width: 1200px) 1100px, 92vw"
        >
        <img
          src={src("light")}
          alt={screenshot.alt}
          width="1920"
          height={screenshot.height * 2}
          loading="eager"
          decoding="async"
        >
      </picture>
    </div>
  </section>

  <!-- Features -->
  <section class="features">
    <div class="feature">
      <div class="feature-icon"><Package class="h-6 w-6" /></div>
      <h2>Package ecosystem</h2>
      <p>
        Ethereum nodes install as packages. Each install is secure, consistent
        across systems, and atomic.
      </p>
    </div>

    <div class="feature">
      <div class="feature-icon"><Layers class="h-6 w-6" /></div>
      <h2>One core, every surface</h2>
      <p>
        The CLI and web service share the same Rust core. Keep the surface area
        small without splitting the logic.
      </p>
    </div>

    <div class="feature">
      <div class="feature-icon"><Wifi class="h-6 w-6" /></div>
      <h2>Remote access</h2>
      <p>
        Monitor and manage your node from anywhere. Upgrade from your phone when
        it matters.
      </p>
    </div>

    <div class="feature">
      <div class="feature-icon"><Lock class="h-6 w-6" /></div>
      <h2>Minimal by default</h2>
      <p>
        Tiny, auditable codebase. Capabilities are opt-in. Read-only mode until
        you need more.
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
    gap: 1.25rem;
    align-items: center;
    text-align: center;
  }

  .hero h1 {
    font-size: clamp(2.1rem, 5.25vw, 3.25rem);
    font-weight: 500;
    line-height: 1.1;
    letter-spacing: -0.02em;
    text-wrap: balance;
  }

  .subtitle {
    font-size: clamp(1.1rem, 1.9vw, 1.3rem);
    line-height: 1.6;
    color: var(--muted-foreground);
  }

  .cta-group {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    align-items: center;
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
    gap: 1.25rem;
    align-items: center;
    width: 100vw;
    padding-inline: max(1.5rem, calc(50vw - 600px));
    margin-inline: calc(50% - 50vw);
  }

  .screenshot-wrapper {
    width: 100%;
    max-width: 1040px;
    overflow: hidden;
    background: var(--muted);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow:
      0 5px 22px rgb(0 0 0 / 0.065),
      0 0 0 1px rgb(0 0 0 / 0.02);
  }

  .screenshot-wrapper img {
    display: block;
    width: 100%;
    height: auto;
  }

  /* Features */
  .features {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 2rem 2.5rem;
    max-width: 800px;
    padding-inline: 1rem;
    margin-inline: auto;
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
    color: var(--primary);
    background: var(--muted);
    border-radius: var(--radius);
  }

  .feature h2 {
    font-family: "Space Grotesk Variable", sans-serif;
    font-size: 1.125rem;
    font-weight: 600;
    letter-spacing: -0.01em;
  }

  .feature p {
    font-size: 1rem;
    line-height: 1.6;
    color: var(--muted-foreground);
  }

  /* Bottom CTA */
  .bottom-cta {
    display: flex;
    justify-content: center;
  }

  .docs-link {
    display: inline-flex;
    gap: 0.5rem;
    align-items: center;
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

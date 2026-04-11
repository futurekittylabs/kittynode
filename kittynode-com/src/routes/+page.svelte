<script lang="ts">
import { Layers, Lock, Package, Wifi } from "@lucide/svelte";
import cliRelease from "$lib/cli-release.json";
import * as Code from "$lib/components/ui/code";

const { version: cliVersion, date: cliPubDate } = cliRelease;

const baseUrl = "https://github.com/futurekittylabs/kittynode";
const changelogUrl = `${baseUrl}/releases`;

const releaseDateFormatter = new Intl.DateTimeFormat("en-US", {
  month: "long",
  day: "numeric",
  year: "numeric",
});

function formatReleaseDate(pubDate: string | undefined): string {
  if (!pubDate) return "";
  const parsed = new Date(pubDate);
  if (Number.isNaN(parsed.getTime())) return "";
  return releaseDateFormatter.format(parsed);
}

const cliReleaseDate = formatReleaseDate(cliPubDate);
const cliVersionLabel = `Version ${cliVersion}`;

const installCommand = `curl --proto '=https' --tlsv1.2 -LsSf https://kittynode.com/sh | sh`;
</script>

<div class="page">
  <!-- Hero -->
  <section class="section">
    <h1>Run Ethereum at home</h1>
    <p class="subtitle">
      Kittynode is a control center for world computer operators.
    </p>
  </section>

  <!-- Install -->
  <section class="section">
    <p class="install-heading">Run Ethereum with a single command:</p>
    <div class="install">
      <Code.Root
        lang="bash"
        code={installCommand}
        hideLines
      >
        <Code.CopyButton variant="secondary" />
      </Code.Root>
    </div>
    <p class="version">
      <a href={changelogUrl} class="link">{cliVersionLabel}</a>
      {#if cliReleaseDate}
        — {cliReleaseDate}
      {/if}
    </p>
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
        The CLI and server share the same Rust core. Keep the surface area small
        without splitting the logic.
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
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: clamp(4rem, 10vw, 6rem);
    padding-block: clamp(3rem, 8vw, 5rem);
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    align-items: center;
    text-align: center;
  }

  .section h1 {
    font-size: clamp(1.85rem, 4.5vw, 2.75rem);
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

  .install-heading {
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 0.875rem;
    color: var(--muted-foreground);
  }

  .version {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--muted-foreground);
  }

  .install {
    max-width: 100%;
  }

  .install :global(pre .line) {
    padding-right: 3.5rem;
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
</style>

<script lang="ts">
import "../app.css";
import { ModeWatcher } from "mode-watcher";

let { children } = $props();
</script>

<ModeWatcher />

<svelte:head>
  <link rel="icon" type="image/x-icon" href="/favicon.ico" />
  <link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png" />
  <link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png" />
  <link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png" />
  <title>Kittynode | Operate the world computer</title>
  <meta property="og:site_name" content="Kittynode" />
  <meta
    name="description"
    content="Control center for world computer operators."
  />
</svelte:head>

<div class="flex min-h-screen flex-col">
  <div class="mx-auto flex min-h-screen w-full max-w-[100ch] flex-col px-6">
    <header class="flex h-16 items-center justify-between border-b px-1">
      <a href="/" class="wordmark">
        <picture>
          <source
            type="image/webp"
            srcset="/kittynode-logo-app-no-padding-160.webp"
          />
          <img
            src="/kittynode-logo-app-no-padding-160.png"
            alt="Kittynode logo"
            class="app-logo"
            width="40"
            height="40"
            decoding="async"
          />
        </picture>
        <span class="wordmark-text">Kittynode</span>
      </a>

      <nav class="flex items-center gap-8">
        <a href="https://docs.kittynode.com" class="nav-link">
          Docs
        </a>
        <a href="/store" class="nav-link">
          Store
        </a>
      </nav>
    </header>

    <main class="flex-1">
      {@render children?.()}
    </main>

    <footer class="border-t py-6">
      <div class="flex flex-col items-center gap-4">
        <div class="flex items-center gap-8 text-sm">
          <a href="https://discord.kittynode.com" class="nav-link">
            Discord
          </a>
          <a href="https://farcaster.xyz/kittynode" class="nav-link">
            Farcaster
          </a>
          <a href="https://github.com/futurekittylabs/kittynode" class="nav-link">
            GitHub
          </a>
          <a href="https://x.com/kittynode" class="nav-link">
            X
          </a>
        </div>
        <div class="text-center text-sm text-muted-foreground">
          Kittynode is <a
            href="https://github.com/futurekittylabs/kittynode"
            class="link">free software</a
          > released under the MIT License.
        </div>
      </div>
    </footer>
  </div>
</div>

<style>
  .wordmark {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    text-decoration: none;
    flex-shrink: 0;
  }

  .app-logo {
    height: 2.5rem;
    width: 2.5rem;
    min-height: 2.5rem;
    min-width: 2.5rem;
    max-width: none;
    flex-shrink: 0;
  }

  .wordmark-text {
    font-size: 1.5rem;
    font-weight: 500;
  }
</style>

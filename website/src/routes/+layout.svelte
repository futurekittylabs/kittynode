<script lang="ts">
import "../app.css";
import { page } from "$app/state";
import { ModeWatcher } from "mode-watcher";
import { onMount } from "svelte";

let { children } = $props();

// Christmas snowfall
onMount(() => {
  const chars = ["❄", "❅", "❆"];
  const el = document.createElement("div");
  el.id = "snow";
  el.style.cssText =
    "position:fixed;inset:0;pointer-events:none;z-index:9999;overflow:hidden";
  el.ariaHidden = "true";
  document.body.appendChild(el);

  const spawn = () => {
    const f = document.createElement("span");
    const fall = 12 + Math.random() * 10;
    const sway = 2 + Math.random() * 3;
    const dx = (Math.random() > 0.5 ? 1 : -1) * (8 + Math.random() * 20);
    f.textContent = chars[(Math.random() * chars.length) | 0];
    f.style.cssText = `
      position:absolute;
      top:-20px;
      left:${Math.random() * 100}%;
      font-size:${0.7 + Math.random()}rem;
      color:#b4d7ff;
      opacity:${0.4 + Math.random() * 0.4};
      animation:snowfall ${fall}s linear forwards,drift ${sway}s ease-in-out ${-Math.random() * sway}s infinite;
      --dx:${dx}px;
    `;
    el.appendChild(f);
    setTimeout(() => f.remove(), fall * 1000);
  };

  for (let i = 0; i < 25; i++) setTimeout(spawn, i * 120);
  const id = setInterval(spawn, 350);
  return () => {
    clearInterval(id);
    el.remove();
  };
});
</script>

<ModeWatcher />

<svelte:head>
  <link rel="icon" type="image/x-icon" href="/favicon.ico" />
  <link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png" />
  <link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png" />
  <link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png" />
  <title>Kittynode | Run the world computer</title>
  <meta property="og:site_name" content="Kittynode" />
  <meta
    name="description"
    content="Control center for world computer operators."
  />
</svelte:head>

<div class="flex min-h-screen flex-col">
  <div class="mx-auto flex min-h-screen w-full max-w-[100ch] flex-col px-6">
    <header class="flex h-16 items-center justify-between border-b px-0.5">
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

      <nav class="flex items-center gap-6">
        <a href="https://docs.kittynode.com/start-here/getting-started" class="nav-link">
          Docs
        </a>
        <a
          href="/download"
          class="nav-link"
          class:text-link={page.url.pathname === "/download"}
          aria-current={page.url.pathname === "/download" ? "page" : undefined}
        >
          Download
        </a>
      </nav>
    </header>

    <main class="flex-1">
      {@render children?.()}
    </main>

    <footer class="border-t py-6">
      <div class="flex flex-col items-center gap-5">
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
    height: 2.25rem;
    width: 2.25rem;
    min-height: 2.25rem;
    min-width: 2.25rem;
    max-width: none;
    flex-shrink: 0;
  }

  .wordmark-text {
    font-size: 1.5rem;
    font-weight: 500;
  }
</style>

<script lang="ts">
import { page } from "$app/state";
import { House, HeartPulse, Settings } from "@lucide/svelte";
</script>

<nav>
  <div class="flex h-5 items-center justify-between text-sm font-semibold">
    <a
      class="p-4 ps-0 {page.url.pathname === '/' ? 'text-primary' : ''}"
      href="/"
      aria-current={page.url.pathname === "/"}
    >
      <House strokeWidth={page.url.pathname === "/" ? 2.25 : 2} />
    </a>
    <a
      class="p-4 {page.url.pathname === '/system-info' ? 'text-primary' : ''}"
      href="/system-info"
      aria-current={page.url.pathname === "/system-info"}
    >
      <HeartPulse strokeWidth={page.url.pathname === "/system-info" ? 2.25 : 2} />
    </a>
    <a
      class="p-4 pe-0 {page.url.pathname === '/settings' ? 'text-primary' : ''}"
      href="/settings"
      aria-current={page.url.pathname === "/settings"}
    >
      <Settings strokeWidth={page.url.pathname === "/settings" ? 2.25 : 2} />
    </a>
  </div>
</nav>

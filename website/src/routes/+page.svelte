<script lang="ts">
import { ArrowRight } from "@lucide/svelte";
import { Button } from "$lib/components/ui/button/index.js";
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
            <Button href="/download" size="lg" class="gap-2">
              Get started
              <ArrowRight class="h-5 w-5" />
            </Button>
            <p class="text-sm text-muted-foreground">
              Available for Linux, macOS, and Windows.
            </p>
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

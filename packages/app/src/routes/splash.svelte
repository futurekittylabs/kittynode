<script lang="ts">
  import { ArrowUpRight, TriangleAlert } from "@lucide/svelte";
  import { platform } from "@tauri-apps/plugin-os";
  import { mode } from "mode-watcher";
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";
  import { goto } from "$app/navigation";
  import { coreClient } from "$lib/client";
  import { Button } from "$lib/components/ui/button";
  import { Progress } from "$lib/components/ui/progress";
  import { initializedState } from "$lib/states/initialized.svelte";

  let currentPlatform = $state("");
  let currentStep = $state(0);

  const totalSteps = 4;
  const isFirstStep = $derived(currentStep === 0);
  const isLastStep = $derived(currentStep === totalSteps - 1);
  const progressValue = $derived(
    totalSteps <= 1 ? 100 : (currentStep / (totalSteps - 1)) * 100
  );
  const isInitializing = $derived(initializedState.initializing);

  let canvasElement: HTMLCanvasElement;
  let animationFrameId: number;
  let ctx: CanvasRenderingContext2D;

  interface NodePoint {
    x: number;
    y: number;
    vx: number;
    vy: number;
  }

  onMount(() => {
    currentPlatform = platform();

    const canvas = canvasElement;
    const context = canvas?.getContext("2d");
    if (!context) {
      console.error("Please report this error: Failed to get 2D context.");
      return;
    }
    ctx = context;

    const nodes: NodePoint[] = [];
    const maxVelocity = 1.2;
    const nodeDensity = 0.000_08;

    function calculateNumNodes() {
      return Math.round(window.innerWidth * window.innerHeight * nodeDensity);
    }

    function resizeCanvas() {
      const dpr = window.devicePixelRatio || 1;
      canvas.width = window.innerWidth * dpr;
      canvas.height = window.innerHeight * dpr;
      canvas.style.width = `${window.innerWidth}px`;
      canvas.style.height = `${window.innerHeight}px`;
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

      const newNumNodes = calculateNumNodes();
      if (newNumNodes > nodes.length) {
        const nodesToAdd = newNumNodes - nodes.length;
        for (let i = 0; i < nodesToAdd; i++) {
          nodes.push({
            x: Math.random() * window.innerWidth,
            y: Math.random() * window.innerHeight,
            vx: (Math.random() - 0.5) * maxVelocity,
            vy: (Math.random() - 0.5) * maxVelocity,
          });
        }
      } else if (newNumNodes < nodes.length) {
        nodes.splice(0, nodes.length - newNumNodes);
      }

      for (const node of nodes) {
        if (node.x > window.innerWidth) {
          node.x = Math.random() * window.innerWidth;
        }
        if (node.y > window.innerHeight) {
          node.y = Math.random() * window.innerHeight;
        }
      }
    }

    resizeCanvas();
    window.addEventListener("resize", resizeCanvas);

    function animate() {
      ctx.fillStyle = mode.current === "dark" ? "#09090b" : "#f5f5f7";
      ctx.fillRect(0, 0, canvas.width, canvas.height);

      for (const node of nodes) {
        node.x += node.vx;
        node.y += node.vy;
        if (node.x <= 0 || node.x >= window.innerWidth) {
          node.vx *= -1;
        }
        if (node.y <= 0 || node.y >= window.innerHeight) {
          node.vy *= -1;
        }

        ctx.beginPath();
        ctx.arc(node.x, node.y, 1.8, 0, Math.PI * 2);
        ctx.fillStyle = mode.current === "dark" ? "#FFFFFF" : "#111111";
        ctx.fill();
      }

      for (let i = 0; i < nodes.length; i++) {
        for (let j = i + 1; j < nodes.length; j++) {
          const dx = nodes[i].x - nodes[j].x;
          const dy = nodes[i].y - nodes[j].y;
          const distance = Math.hypot(dx, dy);

          if (distance < 110) {
            ctx.beginPath();
            ctx.moveTo(nodes[i].x, nodes[i].y);
            ctx.lineTo(nodes[j].x, nodes[j].y);
            const color = mode.current === "dark" ? "255,255,255" : "17,17,17";
            ctx.strokeStyle = `rgba(${color}, ${1 - distance / 110})`;
            ctx.lineWidth = 0.9;
            ctx.stroke();
          }
        }
      }

      animationFrameId = requestAnimationFrame(animate);
    }

    animate();

    return () => {
      cancelAnimationFrame(animationFrameId);
      window.removeEventListener("resize", resizeCanvas);
    };
  });

  function nextStep() {
    if (!isLastStep) {
      currentStep += 1;
    }
  }

  function prevStep() {
    if (!isFirstStep) {
      currentStep -= 1;
    }
  }

  async function initKittynode() {
    try {
      if (["ios", "android"].includes(currentPlatform)) {
        await initializedState.fakeInitialize();
      } else {
        await initializedState.initialize();
      }
      try {
        await coreClient.setOnboardingCompleted(true);
      } catch (e) {
        console.error(`Failed to save onboarding state: ${e}`);
        toast.error("Failed to save onboarding progress");
      }
    } catch (e) {
      console.error(`Failed to initialize kittynode: ${e}`);
      toast.error("Failed to initialize Kittynode");
      return;
    }

    await goto("/");
  }
</script>

<canvas class="fixed inset-0 -z-10" bind:this={canvasElement}></canvas>

<main
  class="relative z-10 flex min-h-svh w-full items-center justify-center px-6 py-10 sm:px-10 sm:py-16"
>
  <section
    class="flex w-full max-w-4xl flex-col rounded-4xl border border-border/40 bg-background/90 px-6 py-10 shadow-none backdrop-blur-sm sm:px-12 sm:py-14"
  >
    <header class="space-y-6 kittynode-onboard-font">
      <div class="flex flex-wrap items-center gap-3">
        <div class="flex items-center gap-3">
          <img
            src="/images/kittynode-logo-app-no-padding.png"
            alt="Kittynode Logo"
            class="h-12 w-12"
          >
          <span
            class="kittynode-brand text-[1.9rem] leading-none sm:text-[2.1rem]"
            >Kittynode</span
          >
        </div>
      </div>
      <Progress value={progressValue} class="h-1" />
    </header>

    <div class="kittynode-onboard-font mt-8 min-h-60">
      {#if currentStep === 0}
        <div class="flex h-full flex-col justify-center gap-6 text-left">
          <div class="space-y-4">
            <h1
              class="font-medium leading-tight text-[1.9rem] text-foreground sm:text-[2rem]"
            >
              Welcome to Kittynode
            </h1>
            <p class="text-base text-muted-foreground sm:text-lg">
              Run the world computer. Secure the Ethereum network and earn
              rewards.
            </p>
          </div>
        </div>
      {:else if currentStep === 1}
        <div class="flex h-full flex-col justify-center gap-6 text-left">
          <div class="space-y-4">
            <h2
              class="font-medium leading-tight text-[1.9rem] text-foreground sm:text-[2rem]"
            >
              Try Kittynode CLI
            </h2>
            <p class="text-base text-muted-foreground sm:text-lg">
              Install Ethereum on a dedicated machine using Kittynode CLI, and
              connect to your installation from anywhere with the Kittynode app!
            </p>
            <p class="text-base text-muted-foreground sm:text-lg">
              This requires both machines to be in the same Tailscale network.
            </p>
          </div>
        </div>
      {:else if currentStep === 2}
        <div class="flex h-full flex-col justify-center gap-6 text-left">
          <div class="space-y-4">
            <h2
              class="font-medium leading-tight text-[1.9rem] text-foreground sm:text-[2rem]"
            >
              Install Docker
            </h2>
            <p class="text-base text-muted-foreground sm:text-lg">
              Kittynode isolates your node stack inside Docker, so you'll need
              it to install Ethereum. Install Docker on whatever machine(s) you
              plan to run Ethereum on.
            </p>
            <p class="text-base text-muted-foreground sm:text-lg">
              If using a remote dedicated machine, you only need to install
              Docker there.
            </p>
          </div>
        </div>
      {:else}
        <div class="flex h-full flex-col justify-center gap-6 text-left">
          <div class="space-y-4">
            <div class="flex items-center gap-3">
              <span
                class="inline-flex h-8 w-8 items-center justify-center rounded-full bg-amber-100 text-amber-700 sm:h-9 sm:w-9"
              >
                <TriangleAlert class="h-5 w-5" aria-hidden="true" />
              </span>
              <h2
                class="font-medium leading-tight text-[1.9rem] text-foreground sm:text-[2rem]"
              >
                Warning
              </h2>
            </div>
            <div class="space-y-4 text-base text-muted-foreground sm:text-lg">
              <p>
                Kittynode has <strong>not been audited</strong>, and may not be
                using audited subcomponents at this time. It is
                <strong
                  >not recommended for mainnet validators</strong
                >. For guidance on mainnet validators please visit
                <a
                  class="link inline-flex items-center gap-1 font-medium"
                  href="https://ethereum.org/staking/solo"
                  rel="noreferrer noopener"
                  target="_blank"
                >
                  ethereum.org/staking/solo
                  <ArrowUpRight class="h-4 w-4" />
                </a>.
              </p>
              <p>Thank you for giving Kittynode a try.</p>
            </div>
          </div>
        </div>
      {/if}
    </div>

    <footer
      class="kittynode-onboard-font mt-12 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between"
    >
      <div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:gap-3">
        <span class="text-sm text-muted-foreground">
          Step {currentStep + 1} of {totalSteps}
        </span>
        {#if currentStep === 1}
          <a
            class="link inline-flex items-center gap-1 text-base font-medium"
            href="https://kittynode.com/download"
            target="_blank"
            rel="noreferrer noopener"
          >
            Download kittynode-cli
            <ArrowUpRight class="h-4 w-4" />
          </a>
        {:else if currentStep === 2}
          <a
            class="link inline-flex items-center gap-1 text-base font-medium"
            href="https://docs.kittynode.com/guides/set-up-docker"
            target="_blank"
            rel="noreferrer noopener"
          >
            Set up Docker
            <ArrowUpRight class="h-4 w-4" />
          </a>
        {/if}
      </div>
      <div class="flex justify-end gap-2">
        <Button
          variant="outline"
          onclick={prevStep}
          disabled={isFirstStep || isInitializing}
          class="min-w-[104px]"
        >
          Back
        </Button>
        {#if isLastStep}
          <Button
            onclick={initKittynode}
            disabled={isInitializing}
            class="min-w-[104px]"
          >
            {#if isInitializing}
              Launching...
            {:else}
              Launch
            {/if}
          </Button>
        {:else}
          <Button onclick={nextStep} class="min-w-[104px]">Next</Button>
        {/if}
      </div>
    </footer>
  </section>
</main>

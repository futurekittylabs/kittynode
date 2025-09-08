<script lang="ts">
import { initializedStore } from "$stores/initialized.svelte";
import { goto } from "$app/navigation";
import { platform } from "@tauri-apps/plugin-os";
import { onMount } from "svelte";
import { mode } from "mode-watcher";
import { Button } from "$lib/components/ui/button";
import * as Card from "$lib/components/ui/card";

let currentPlatform = $state("");
let canvasElement: HTMLCanvasElement;
let animationFrameId: number;
let ctx: CanvasRenderingContext2D;

async function initKittynode() {
  try {
    if (["ios", "android"].includes(currentPlatform)) {
      await initializedStore.fakeInitialize();
    } else {
      await initializedStore.initialize();
    }
  } catch (e) {
    console.error(`Failed to initialize kittynode: ${e}`);
  }
  await goto("/");
}

interface Node {
  x: number;
  y: number;
  vx: number;
  vy: number;
}

onMount(() => {
  currentPlatform = platform();

  const canvas = canvasElement;
  const context = canvas.getContext("2d");
  if (!context) {
    console.error("Please report this error: Failed to get 2D context.");
    return;
  }
  ctx = context;

  const nodes: Node[] = [];
  const maxVelocity = 1.5;
  const nodeDensity = 0.0001;

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
      if (node.x > window.innerWidth)
        node.x = Math.random() * window.innerWidth;
      if (node.y > window.innerHeight)
        node.y = Math.random() * window.innerHeight;
    }
  }

  resizeCanvas(); // Initialize canvas dimensions and nodes
  window.addEventListener("resize", resizeCanvas);

  function animate() {
    ctx.fillStyle = mode.current === "dark" ? "#0a0a0a" : "#FFFFFF";
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    for (const node of nodes) {
      node.x += node.vx;
      node.y += node.vy;
      if (node.x <= 0 || node.x >= window.innerWidth) node.vx *= -1;
      if (node.y <= 0 || node.y >= window.innerHeight) node.vy *= -1;

      ctx.beginPath();
      ctx.arc(node.x, node.y, 2, 0, Math.PI * 2);
      ctx.fillStyle = mode.current === "dark" ? "#FFFFFF" : "#000000";
      ctx.fill();
    }

    for (let i = 0; i < nodes.length; i++) {
      for (let j = i + 1; j < nodes.length; j++) {
        const dx = nodes[i].x - nodes[j].x;
        const dy = nodes[i].y - nodes[j].y;
        const distance = Math.hypot(dx, dy);

        if (distance < 100) {
          ctx.beginPath();
          ctx.moveTo(nodes[i].x, nodes[i].y);
          ctx.lineTo(nodes[j].x, nodes[j].y);
          const color = mode.current === "dark" ? "255, 255, 255" : "0, 0, 0";
          ctx.strokeStyle = `rgba(${color}, ${1 - distance / 100})`;
          ctx.lineWidth = 1;
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
</script>

<canvas class="fixed -z-10 top-0 left-0" bind:this={canvasElement}></canvas>

<main class="relative z-[1] flex flex-col justify-center items-center h-full text-center p-4">
  <Card.Root class="min-w-80 max-w-md mx-4">
    <Card.Header class="flex flex-col items-center">
      <img
      class="logo w-24"
      src={`/images/kittynode-logo-circle.png`}
      alt="Kittynode Logo"
    />
      <Card.Title>Kittynode</Card.Title>
      <Card.Description>
        <div class="mt-3">Control center for world computer operators.</div>
      </Card.Description>
    </Card.Header>
    <Card.Content>
      <Button
        onclick={initKittynode}
        class="transition-transform hover:scale-105 active:scale-100"
      >
        Get Started
      </Button>
    </Card.Content>
  </Card.Root>
</main>


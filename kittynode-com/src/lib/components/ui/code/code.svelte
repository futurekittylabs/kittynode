<!--
	Installed from @ieedan/shadcn-svelte-extras
-->

<script lang="ts">
import { box } from "svelte-toolbelt";
import { cn } from "$lib/utils";
import { codeVariants } from ".";
import { useCode } from "./code.svelte.js";
import type { CodeRootProps } from "./types";

let {
  ref = $bindable(null),
  variant = "default",
  lang = "typescript",
  code,
  class: className,
  hideLines = false,
  highlight = [],
  children,
  ...rest
}: CodeRootProps = $props();

const codeState = useCode({
  code: box.with(() => code),
  hideLines: box.with(() => hideLines),
  highlight: box.with(() => highlight),
  lang: box.with(() => lang),
});
</script>

<div {...rest} bind:this={ref} class={cn(codeVariants({ variant }), className)}>
  {@html codeState.highlighted}
  {@render children?.()}
</div>

<style>
  :global(.dark) {
    :global(.shiki),
    :global(.shiki span) {
      font-style: var(--shiki-dark-font-style) !important;
      font-weight: var(--shiki-dark-font-weight) !important;
      color: var(--shiki-dark) !important;
      text-decoration: var(--shiki-dark-text-decoration) !important;
    }
  }

  /* Shiki see: https://shiki.matsu.io/guide/dual-themes#class-based-dark-mode */
  :global(html.dark .shiki),
  :global(html.dark .shiki span) {
    font-style: var(--shiki-dark-font-style) !important;
    font-weight: var(--shiki-dark-font-weight) !important;
    color: var(--shiki-dark) !important;
    text-decoration: var(--shiki-dark-text-decoration) !important;
  }

  :global(pre.shiki) {
    padding-top: 1rem;
    padding-bottom: 1rem;
    overflow-x: auto;
    font-size: 0.875rem;
    line-height: 1.25rem;
    background-color: inherit;
    border-radius: 0.5rem;
  }

  :global(pre.shiki:not([data-code-overflow] *):not([data-code-overflow])) {
    max-height: min(100%, 650px);
    overflow-y: auto;
  }

  :global(pre.shiki code) {
    display: grid;
    min-width: 100%;
    border-radius: 0;
    border-width: 0;
    background-color: transparent;
    padding: 0;
    overflow-wrap: break-word;
    counter-reset: line;
    box-decoration-break: clone;
  }

  :global(pre.line-numbers) {
    counter-reset: step;
    counter-increment: step 0;
  }

  :global(pre.line-numbers .line::before) {
    display: inline-block;
    width: 1.8rem;
    margin-right: 1.4rem;
    text-align: right;
    content: counter(step);
    counter-increment: step;
  }

  :global(pre.line-numbers .line::before) {
    color: var(--muted-foreground);
  }

  :global(pre .line.line--highlighted) {
    background-color: var(--secondary);
  }

  :global(pre .line.line--highlighted span) {
    position: relative;
  }

  :global(pre .line) {
    display: inline-block;
    width: 100%;
    min-height: 1rem;
    padding-top: 0.125rem;
    padding-right: 1rem;
    padding-bottom: 0.125rem;
    padding-left: 1rem;
  }

  :global(pre.line-numbers .line) {
    padding-right: 0.5rem;
    padding-left: 0.5rem;
  }
</style>

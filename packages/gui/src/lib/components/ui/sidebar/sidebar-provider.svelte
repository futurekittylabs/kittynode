<script lang="ts">
import { setContext } from "svelte";
import { writable } from "svelte/store";

type SidebarState = {
  isOpen: boolean;
  isMobile: boolean;
};

let {
  defaultOpen = true,
  children,
}: {
  defaultOpen?: boolean;
  children?: any;
} = $props();

const state = writable<SidebarState>({
  isOpen: defaultOpen,
  isMobile: false,
});

function toggle() {
  state.update((s) => ({ ...s, isOpen: !s.isOpen }));
}

function setOpen(value: boolean) {
  state.update((s) => ({ ...s, isOpen: value }));
}

function setMobile(value: boolean) {
  state.update((s) => ({ ...s, isMobile: value }));
}

setContext("sidebar", {
  state,
  toggle,
  setOpen,
  setMobile,
});

$effect(() => {
  const checkMobile = () => {
    const isMobile = window.innerWidth < 768;
    setMobile(isMobile);
    if (isMobile) {
      setOpen(false);
    }
  };

  checkMobile();
  window.addEventListener("resize", checkMobile);

  return () => {
    window.removeEventListener("resize", checkMobile);
  };
});
</script>

{@render children?.()}

<script lang="ts">
import * as Card from "$lib/components/ui/card";
import {
  dockerStatus,
  type DockerStatusValue,
} from "$stores/dockerStatus.svelte";
import {
  CircleCheck,
  CircleAlert,
  CircleX,
  Loader2,
  Server,
} from "@lucide/svelte";

const props = $props<{ showServerIcon?: boolean }>();
const showServerIcon = $derived(props.showServerIcon ?? false);

type StatusDescriptor = {
  label: string;
  icon: typeof CircleCheck;
  className: string;
  spinning?: boolean;
};

const statusCopy: Record<DockerStatusValue, StatusDescriptor> = {
  running: { label: "Running", icon: CircleCheck, className: "text-green-500" },
  starting: {
    label: "Starting Docker Desktop",
    icon: Loader2,
    className: "text-blue-500",
    spinning: true,
  },
  not_installed: {
    label: "Docker Desktop not installed",
    icon: CircleX,
    className: "text-red-500",
  },
  not_running: {
    label: "Docker Desktop not running",
    icon: CircleAlert,
    className: "text-yellow-500",
  },
  unknown: {
    label: "Checking Docker status",
    icon: Loader2,
    className: "text-muted-foreground",
    spinning: true,
  },
};

const statusDescriptor = $derived(
  statusCopy[dockerStatus.status] ?? statusCopy.unknown,
);
</script>

<Card.Root>
  <Card.Header class="pb-3">
    <Card.Title class="text-sm font-medium flex items-center justify-between">
      Docker Status
      {#if showServerIcon}
        <Server class="h-4 w-4 text-muted-foreground" />
      {/if}
    </Card.Title>
  </Card.Header>
  <Card.Content>
    {@const Icon = statusDescriptor.icon}
    <div class="flex items-center space-x-2">
      <Icon
        class={`h-4 w-4 ${statusDescriptor.className} ${statusDescriptor.spinning ? "animate-spin" : ""}`}
      />
      <span class="text-sm font-medium">{statusDescriptor.label}</span>
    </div>
  </Card.Content>
</Card.Root>

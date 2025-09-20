<script lang="ts">
import {
  Terminal,
  Monitor,
  Download,
  AppWindowMac,
  CircleQuestionMark,
  ChevronDown,
} from "@lucide/svelte";
import { Button } from "$lib/components/ui/button/index.js";
import {
  Collapsible,
  CollapsibleTrigger,
  CollapsibleContent,
} from "$lib/components/ui/collapsible/index.js";
import releaseInfo from "$lib/release.json";

const { version, date: releaseDate } = releaseInfo;

const baseUrl = "https://github.com/futurekittylabs/kittynode";
const releaseUrl = `${baseUrl}/releases/download/kittynode-app@${version}`;
const discordUrl = "https://discord.kittynode.com";

const downloads = [
  {
    name: "Linux",
    icon: Terminal,
    requirements: "Linux (x86_64)",
    options: [
      {
        label: ".deb",
        url: `${releaseUrl}/Kittynode_${version}_amd64.deb`,
      },
      {
        label: ".rpm",
        url: `${releaseUrl}/Kittynode-${version}-1.x86_64.rpm`,
      },
    ],
    layout: "stacked",
  },
  {
    name: "macOS",
    icon: AppWindowMac,
    requirements: "macOS 10.15+ (Apple Silicon)",
    primary: {
      label: ".dmg",
      url: `${releaseUrl}/Kittynode_${version}_aarch64.dmg`,
    },
    options: [
      {
        label: ".app.tar.gz",
        url: `${releaseUrl}/Kittynode_aarch64.app.tar.gz`,
      },
    ],
  },
  {
    name: "Windows",
    icon: Monitor,
    requirements: "Windows 7+ (x86_64)",
    primary: {
      label: ".exe",
      url: `${releaseUrl}/Kittynode_${version}_x64-setup.exe`,
    },
    options: [
      {
        label: ".msi",
        url: `${releaseUrl}/Kittynode_${version}_x64_en-US.msi`,
      },
    ],
  },
];

let linuxHelpOpen = false;
</script>

<div class="container max-w-6xl mx-auto px-6 py-16">
	<!-- Header -->
	<div class="mb-8 text-center">
		<h1 class="text-3xl font-medium mb-2">Download Kittynode</h1>
		<p class="text-sm text-muted-foreground mb-4">Version {version} • {releaseDate}</p>
		<a href="{baseUrl}/releases" class="link text-sm">
			View changelog
		</a>
	</div>
	<!-- Download cards -->
	<div class="grid gap-4 min-[900px]:grid-cols-3">
		{#each downloads as info}
			<div class="rounded-lg border bg-card p-5">
				<div class="flex items-center gap-3 mb-4">
					<div class="p-1.5 rounded-md bg-muted">
						<svelte:component this={info.icon} class="h-5 w-5" />
					</div>
					<div>
						<h2 class="text-base font-medium">{info.name}</h2>
						<p class="text-xs text-muted-foreground">{info.requirements}</p>
					</div>
				</div>

				<div class="space-y-2">
					{#if info.primary}
						<Button href={info.primary.url} size="sm" class="w-full gap-2" variant="default">
							<Download class="h-3.5 w-3.5" />
							{info.primary.label}
						</Button>
					{/if}

					{#if info.options && info.options.length > 0}
						{#if info.layout === "stacked"}
							{#each info.options as option}
								<Button href={option.url} size="sm" class="w-full gap-2" variant="outline">
									<Download class="h-3.5 w-3.5" />
									{option.label}
								</Button>
							{/each}
						{:else}
							<div class="flex gap-2">
								{#each info.options as option}
									<Button href={option.url} size="sm" variant="outline" class="flex-1 gap-2">
										<Download class="h-3.5 w-3.5" />
										{option.label}
									</Button>
								{/each}
							</div>
						{/if}
					{/if}
				</div>
			</div>
		{/each}
	</div>


	<Collapsible bind:open={linuxHelpOpen} class="mt-10 max-w-2xl mx-auto">
		<div class="overflow-hidden rounded-lg border">
			<CollapsibleTrigger class="flex w-full items-center justify-between gap-3 px-4 py-3 text-left text-sm font-medium transition-colors hover:bg-muted/60">
				<span class="flex items-center gap-2">
					<CircleQuestionMark class="h-4 w-4 text-link" />
					Looking for another Linux package format?
				</span>
				<ChevronDown class={`h-4 w-4 transition-transform ${linuxHelpOpen ? "rotate-180" : ""}`} />
			</CollapsibleTrigger>
			<CollapsibleContent class="space-y-3 px-4 pb-4 pt-1 text-sm text-muted-foreground">
				<p>
					We're expanding our Linux packaging support beyond the options listed above.
				</p>
				<p>
					Please reach out on <a href={discordUrl} class="link">Discord</a> or <a href={baseUrl} class="link">GitHub</a> if your distro is not supported — we want to support your system and will prioritize it!
				</p>
			</CollapsibleContent>
		</div>
	</Collapsible>

</div>

// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import starlightLinksValidator from "starlight-links-validator";
import starlightImageZoom from "starlight-image-zoom";

// https://astro.build/config
export default defineConfig({
  site: "https://kittynode.io",
  integrations: [
    starlight({
      plugins: [starlightLinksValidator(), starlightImageZoom()],
      title: "Kittynode",
      logo: {
        light: "./src/assets/kittynode-wordmark-light.png",
        dark: "./src/assets/kittynode-wordmark-dark.png",
        replacesTitle: true,
      },
      editLink: {
        baseUrl: "https://github.com/blackkittylabs/kittynode/edit/main/docs/",
      },
      components: {
        Footer: "./src/components/overrides/Footer.astro",
      },
      customCss: ["./src/styles/custom.css"],
      favicon: "/images/favicon.ico",
      social: [
        {
          icon: "github",
          label: "GitHub",
          href: "https://github.com/blackkittylabs/kittynode",
        },
        {
          icon: "discord",
          label: "Discord",
          href: "https://discord.kittynode.io",
        },
        {
          icon: "farcaster",
          label: "Farcaster",
          href: "https://warpcast.com/kittynode",
        },
      ],
      sidebar: [
        {
          label: "Learn",
          items: [
            { label: "What is Kittynode?", slug: "learn/what-is-kittynode" },
            {
              label: "Architecture",
              slug: "learn/architecture",
            },
          ],
        },
        {
          label: "Development",
          items: [
            {
              label: "Development guide",
              slug: "development/development-guide",
            },
            { label: "Releases", slug: "development/releases" },
          ],
        },
        {
          label: "Resources",
          items: [
            {
              label: "Educational materials",
              slug: "resources/educational-materials",
            },
            {
              label: "Hardware resources",
              slug: "resources/hardware-resources",
            },
          ],
        },
      ],
    }),
  ],
});

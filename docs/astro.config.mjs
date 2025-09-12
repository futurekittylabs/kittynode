// @ts-check
import cloudflare from "@astrojs/cloudflare";
import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";
import starlightImageZoom from "starlight-image-zoom";
import starlightLinksValidator from "starlight-links-validator";

// https://astro.build/config
export default defineConfig({
  redirects: {
    "/": "/reference/architecture",
  },
  adapter: cloudflare({
    imageService: "compile",
  }),
  site: "https://docs.kittynode.com",
  integrations: [
    starlight({
      plugins: [starlightLinksValidator(), starlightImageZoom()],
      title: "Kittynode Docs",
      editLink: {
        baseUrl: "https://github.com/blackkittylabs/kittynode/edit/main/docs/",
      },
      components: {
        Footer: "./src/components/overrides/Footer.astro",
        SiteTitle: "./src/components/overrides/SiteTitle.astro",
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
          href: "https://discord.kittynode.com",
        },
        {
          icon: "farcaster",
          label: "Farcaster",
          href: "https://farcaster.xyz/kittynode.eth",
        },
        {
          icon: "x.com",
          label: "X",
          href: "https://x.com/kittynode",
        },
      ],
      sidebar: [
        {
          label: "Reference",
          items: [
            {
              label: "Architecture",
              slug: "reference/architecture",
            },
          ],
        },
      ],
    }),
  ],
});

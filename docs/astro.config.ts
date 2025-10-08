import cloudflare from "@astrojs/cloudflare";
import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";
import starlightImageZoom from "starlight-image-zoom";
import starlightLinksValidator from "starlight-links-validator";

// https://astro.build/config
export default defineConfig({
  redirects: {
    "/": "/guides/build-from-source",
    "/guides/run-ethereum": "/guides/run-remotely",
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
        baseUrl: "https://github.com/futurekittylabs/kittynode/edit/main/docs/",
      },
      components: {
        Footer: "./src/components/overrides/Footer.astro",
        SiteTitle: "./src/components/overrides/SiteTitle.astro",
      },
      customCss: ["./src/styles/custom.css"],
      favicon: "/images/favicon.ico",
      head: [
        {
          tag: "link",
          attrs: {
            rel: "icon",
            type: "image/png",
            sizes: "16x16",
            href: "/images/favicon-16x16.png",
          },
        },
        {
          tag: "link",
          attrs: {
            rel: "icon",
            type: "image/png",
            sizes: "32x32",
            href: "/images/favicon-32x32.png",
          },
        },
        {
          tag: "link",
          attrs: {
            rel: "apple-touch-icon",
            sizes: "180x180",
            href: "/images/apple-touch-icon.png",
          },
        },
      ],
      social: [
        {
          icon: "github",
          label: "GitHub",
          href: "https://github.com/futurekittylabs/kittynode",
        },
        {
          icon: "discord",
          label: "Discord",
          href: "https://discord.kittynode.com",
        },
      ],
      sidebar: [
        {
          label: "Guides",
          items: [
            { label: "Run remotely", slug: "guides/run-remotely" },
            { label: "Install Docker", slug: "guides/install-docker" },
            { label: "Use Tailscale", slug: "guides/tailscale" },
            { label: "Build from source", slug: "guides/build-from-source" },
          ],
        },
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

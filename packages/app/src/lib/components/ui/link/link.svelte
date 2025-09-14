<script lang="ts">
interface Props {
  href: string;
  targetBlank?: boolean;
  text: string;
}

let { href, targetBlank = false, text }: Props = $props();
</script>

<a href={href} target={targetBlank ? "_blank" : undefined } class="text-primary underline underline-offset-4">{text}</a>

import Root from "./alert.svelte";
import Action from "./alert-action.svelte";
import Description from "./alert-description.svelte";
import Title from "./alert-title.svelte";

export { type AlertVariant, alertVariants } from "./alert.svelte";

export {
  Root,
  Description,
  Title,
  Action,
  //
  Root as Alert,
  Description as AlertDescription,
  Title as AlertTitle,
  Action as AlertAction,
};

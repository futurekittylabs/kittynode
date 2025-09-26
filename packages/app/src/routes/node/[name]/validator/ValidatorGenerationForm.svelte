<script lang="ts">
import { Input } from "$lib/components/ui/input";
import { Button } from "$lib/components/ui/button";
import { Switch } from "$lib/components/ui/switch";
import * as Alert from "$lib/components/ui/alert";
import { AlertTriangle } from "@lucide/svelte";

interface Props {
  validatorCount: string;
  password: string;
  passwordConfirm: string;
  withdrawalAddress: string;
  compounding: boolean;
  amount: string | number;
  usePbkdf2: boolean;
  generating: boolean;
  network: string;
  onSubmit: () => void;
}

let {
  validatorCount = $bindable(),
  password = $bindable(),
  passwordConfirm = $bindable(),
  withdrawalAddress = $bindable(),
  compounding = $bindable(),
  amount = $bindable(),
  usePbkdf2 = $bindable(),
  generating,
  network,
  onSubmit,
}: Props = $props();

const isMainnet = $derived(network === "mainnet");
</script>

<div class="space-y-6">
  <Alert.Root variant="default">
    <AlertTriangle class="h-4 w-4" />
    <Alert.Title>Security Notice</Alert.Title>
    <Alert.Description>
      Generate keys on an air-gapped machine for maximum security. Never share your mnemonic phrase.
    </Alert.Description>
  </Alert.Root>

  <div class="space-y-4">
    <div>
      <label for="new-mnemonic-count" class="mb-2 block text-sm font-medium">
        Number of validators
      </label>
      <Input
        id="new-mnemonic-count"
        type="number"
        min="1"
        max="1000"
        bind:value={validatorCount}
        placeholder="1"
        disabled={generating}
      />
    </div>

    <div>
      <label for="new-mnemonic-password" class="mb-2 block text-sm font-medium">
        Keystore password (min 12 characters)
      </label>
      <Input
        id="new-mnemonic-password"
        type="password"
        bind:value={password}
        placeholder="Enter a strong password"
        disabled={generating}
      />
    </div>

    <div>
      <label for="new-mnemonic-password-confirm" class="mb-2 block text-sm font-medium">
        Confirm keystore password
      </label>
      <Input
        id="new-mnemonic-password-confirm"
        type="password"
        bind:value={passwordConfirm}
        placeholder="Re-enter your password"
        disabled={generating}
      />
    </div>

    <div>
      <label for="new-mnemonic-withdrawal" class="mb-2 block text-sm font-medium">
        Withdrawal address (optional, 0x...)
      </label>
      <Input
        id="new-mnemonic-withdrawal"
        type="text"
        bind:value={withdrawalAddress}
        placeholder="0x..."
        disabled={generating}
      />
      <p class="mt-1 text-xs text-muted-foreground">
        Leave empty to use BLS withdrawal credentials (0x00). You can set this later.
      </p>
    </div>

    <div class="flex items-center space-x-2">
      <Switch
        id="new-mnemonic-compounding"
        bind:checked={compounding}
        disabled={generating || !withdrawalAddress}
      />
      <label
        for="new-mnemonic-compounding"
        class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
      >
        Compounding validator (requires withdrawal address)
      </label>
    </div>

    {#if compounding}
      <div>
        <label for="new-mnemonic-amount" class="mb-2 block text-sm font-medium">
          Amount per validator (ETH)
        </label>
        <Input
          id="new-mnemonic-amount"
          type="number"
          min={isMainnet ? 32 : 1}
          step="1"
          bind:value={amount}
          placeholder={isMainnet ? "32" : "1"}
          disabled={generating}
        />
        <p class="mt-1 text-xs text-muted-foreground">
          Minimum: {isMainnet ? "32" : "1"} ETH for {network}
        </p>
      </div>
    {/if}

    <div class="flex items-center space-x-2">
      <Switch
        id="new-mnemonic-pbkdf2"
        bind:checked={usePbkdf2}
        disabled={generating}
      />
      <label
        for="new-mnemonic-pbkdf2"
        class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
      >
        Use PBKDF2 (slower but more secure key derivation)
      </label>
    </div>
  </div>

  <Button
    onclick={onSubmit}
    disabled={generating}
    class="w-full"
  >
    {#if generating}
      <div class="mr-2 h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent"></div>
      Generating...
    {:else}
      Generate keys
    {/if}
  </Button>
</div>
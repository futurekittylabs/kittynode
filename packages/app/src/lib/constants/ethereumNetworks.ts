export const ethereumNetworks = [
  { value: "hoodi", label: "Hoodi" },
  { value: "mainnet", label: "Mainnet" },
  { value: "sepolia", label: "Sepolia" },
  { value: "ephemery", label: "Ephemery" },
] as const;

export const defaultEthereumNetwork = ethereumNetworks[0].value;
export const ethereumNetworkValues = ethereumNetworks.map(
  (network) => network.value,
);

export function formatEthereumNetworks(delimiter: string): string {
  return ethereumNetworkValues.join(delimiter);
}

export function getEthereumNetworkLabel(
  value: string | null | undefined,
): string | null {
  if (!value) {
    return null;
  }

  const trimmed = value.trim();
  if (!trimmed) {
    return null;
  }

  const normalized = trimmed.toLowerCase();
  const match = ethereumNetworks.find(
    (network) => network.value === normalized,
  );
  if (match) {
    return match.label;
  }

  return trimmed[0].toUpperCase() + trimmed.slice(1);
}

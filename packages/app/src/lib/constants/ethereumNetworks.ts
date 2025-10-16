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

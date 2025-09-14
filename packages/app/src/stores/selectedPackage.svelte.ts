import type { Package } from "$lib/types";

let selectedPackage = $state<Package | null>(null);

export const selectedPackageStore = {
  get package() {
    return selectedPackage;
  },

  setPackage(pkg: Package | null) {
    selectedPackage = pkg;
  },

  clear() {
    selectedPackage = null;
  },
};

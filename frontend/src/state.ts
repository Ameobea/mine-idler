import { writable } from 'svelte/store';
import type { ItemDescriptor, LocationDescriptor, UserAccountInfo } from './protos/mine_pb';
import { PrivateClient } from './api';

interface MineLocation {
  descriptor: LocationDescriptor;
  isAvailable: boolean;
}

interface GlobalAppState {
  items: ItemDescriptor[];
  itemsById: Map<number, ItemDescriptor>;
  accountInfo: UserAccountInfo;
  mineLocations?: MineLocation[];
  gambleLocations?: MineLocation[];
}

export const AppLoaded = writable(false);

export const GlobalState = writable<GlobalAppState>({} as any);

export const clearGlobalState = () => {
  AppLoaded.set(false);
  GlobalState.set({} as any);
};

let appStateInitStarted = false;

export const getAppStateInitStarted = () => appStateInitStarted;

export const initAppState = async () => {
  appStateInitStarted = true;
  const [itemDescriptors, accountInfo] = await Promise.all([
    PrivateClient.getItemDescriptors({}).then((res) => res.itemDescriptors),
    PrivateClient.getAccount({}).then((res) => res.userAccountInfo!),
  ]);
  const itemsById = new Map<number, ItemDescriptor>();
  for (const item of itemDescriptors) {
    itemsById.set(item.id, item);
  }

  GlobalState.set({ items: itemDescriptors, itemsById, accountInfo });
  AppLoaded.set(true);

  PrivateClient.getMineLocations({}).then((res) => {
    const mineLocations = res.mineLocations.map((loc) => ({
      descriptor: loc.descriptor!,
      isAvailable: loc.isAvailable,
    }));
    GlobalState.update((state) => ({
      ...state,
      mineLocations,
      gambleLocations: mineLocations, // TODO: Fetch real ones
    }));
  });
};

import { writable } from 'svelte/store';

export enum AppRoute {
  Overview,
  Mine,
  Inventory,
  Hiscores,
}

export const CurRoute = writable<AppRoute>(AppRoute.Overview);

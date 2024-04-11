import type { Renderable, ToastOptions } from 'svelte-french-toast';

declare global {
  declare function dbg<T>(arg: T): T;

  declare function toast(msg: Renderable, options?: ToastOptions): void;
  declare function toastSuccess(msg: Renderable, options?: ToastOptions): void;
  declare function toastError(msg: Renderable, options?: ToastOptions): void;
}

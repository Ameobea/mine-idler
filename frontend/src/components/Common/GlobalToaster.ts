import type { Renderable, ToastOptions } from 'svelte-french-toast';
import { writable } from 'svelte/store';

export const CurToast = writable<{
  message: Renderable;
  options?: ToastOptions;
  variant?: 'success' | 'error';
} | null>(null);

export const initGlobalToaster = () => {
  (window as any).toast = (message: Renderable, options?: ToastOptions) => void CurToast.set({ message, options });
  (window as any).toastSuccess = (message: Renderable, options?: ToastOptions) =>
    void CurToast.set({ message, options, variant: 'success' });
  (window as any).toastError = (message: Renderable, options?: ToastOptions) =>
    void CurToast.set({ message, options, variant: 'error' });
};

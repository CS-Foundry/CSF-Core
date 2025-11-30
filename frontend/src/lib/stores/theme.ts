import { writable, derived, type Unsubscriber } from 'svelte/store';

export type Theme = 'light' | 'dark' | 'system';

// runtime store that holds user's preference: 'light' | 'dark' | 'system'
export const theme = writable<Theme>('system');

// derived store that yields the effective theme (based on system preference when 'system')
export const effectiveTheme = derived(theme, ($theme, set) => {
  let mq: MediaQueryList | null = null;

  const update = () => {
    let prefersDark = false;
    try {
      prefersDark = typeof window !== 'undefined' && window.matchMedia('(prefers-color-scheme: dark)').matches;
    } catch (e) {
      prefersDark = false;
    }
    set($theme === 'system' ? (prefersDark ? 'dark' : 'light') : $theme);
  };

  update();

  if (typeof window !== 'undefined') {
    try {
      // matchMedia returns a MediaQueryList which in older browsers used addListener/removeListener
      mq = window.matchMedia('(prefers-color-scheme: dark)');
      const listener = () => update();
      // modern
      if ((mq as any).addEventListener) (mq as any).addEventListener('change', listener);
      // fallback
      else if ((mq as any).addListener) (mq as any).addListener(listener);

      return () => {
        if (!mq) return;
        if ((mq as any).removeEventListener) (mq as any).removeEventListener('change', listener);
        else if ((mq as any).removeListener) (mq as any).removeListener(listener);
      };
    } catch (e) {
      // ignore
    }
  }

  return undefined as unknown as Unsubscriber;
});

export function setTheme(t: Theme) {
  theme.set(t);
  try { localStorage.setItem('theme', t); } catch (e) { /* ignore */ }
}

export function initThemeFromStorage() {
  if (typeof window === 'undefined') return;
  try {
    const stored = localStorage.getItem('theme') as Theme | null;
    if (stored === 'light' || stored === 'dark' || stored === 'system') theme.set(stored);
  } catch (e) {
    // ignore
  }
}

export function clearStoredTheme() {
  try { localStorage.removeItem('theme'); } catch (e) { /* ignore */ }
  theme.set('system');
}

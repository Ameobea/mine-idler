import * as Sentry from '@sentry/browser';

export const getSentry = (): typeof Sentry | undefined => {
  // Don't clutter up sentry logs with debug stuff
  if (window.location.href.includes('http://localhost')) {
    return undefined;
  }
  return Sentry;
};

export const initSentry = () => {
  // Don't clutter up sentry logs with debug stuff
  if (window.location.href.includes('http://localhost')) {
    return;
  }

  Sentry.init({
    dsn: 'https://fe6f2402504d4e2383ff4566e3676cc5@sentry.ameo.design/6',
    integrations: [Sentry.browserTracingIntegration()],
    tracesSampleRate: 1.0,
  });
};

export const logError = (msg: string, err?: any) => {
  console.error(msg, err);
  const sentry = getSentry();
  if (sentry) {
    if (err) {
      sentry.captureException(err);
    } else {
      sentry.captureMessage(msg, 'error');
    }
  }
};

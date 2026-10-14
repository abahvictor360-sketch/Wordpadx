// The site root used to host the web app, whose service worker cached it here
// and kept serving it to returning visitors. This replacement takes over that
// registration, deletes the old cache, unregisters itself and reloads open
// pages so they show the landing page. The web app now lives under app/ with
// its own service worker and cache.
self.addEventListener('install', () => self.skipWaiting());

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.delete('wordpadx-cache-v1')
      .then(() => self.registration.unregister())
      .then(() => self.clients.matchAll({ type: 'window' }))
      .then((clients) => Promise.all(clients.map((client) => client.navigate(client.url))))
      .catch(() => {})
  );
});

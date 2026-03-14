// sw.js
self.addEventListener('install', (event) => {
    self.skipWaiting();
    console.log('Minimaler Service Worker installiert.');
});

self.addEventListener('activate', (event) => {
  event.waitUntil(clients.claim());
  // Sende Signal an die Seite, dass der SW bereit ist
  self.clients.matchAll().then(clients => {
    clients.forEach(client => client.postMessage({ type: 'SW_READY' }));
  });
});

self.addEventListener('fetch', (event) => {
    event.respondWith(
        fetch(event.request).catch(() => {
            return new Response('Du bist offline.');
        })
    );
});


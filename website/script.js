// Wordpadx landing page: mobile menu, scroll reveal, feature tour, platform filter, theme switch.
const menuToggle = document.getElementById('menuToggle');
const navLinks = document.getElementById('navLinks');

menuToggle?.addEventListener('click', () => {
  const open = navLinks.classList.toggle('is-open');
  menuToggle.setAttribute('aria-expanded', String(open));
});
navLinks?.querySelectorAll('a').forEach((link) => {
  link.addEventListener('click', () => {
    navLinks.classList.remove('is-open');
    menuToggle?.setAttribute('aria-expanded', 'false');
  });
});

// Reveal on scroll.
const revealItems = document.querySelectorAll('.reveal');
if ('IntersectionObserver' in window) {
  const observer = new IntersectionObserver((entries, obs) => {
    entries.forEach((entry) => {
      if (entry.isIntersecting) {
        entry.target.classList.add('is-visible');
        obs.unobserve(entry.target);
      }
    });
  }, { threshold: 0.1 });
  revealItems.forEach((item) => observer.observe(item));
} else {
  revealItems.forEach((item) => item.classList.add('is-visible'));
}

// Feature tour: each item swaps the screenshot in the device.
const tourShot = document.getElementById('tourShot');
const tourItems = document.querySelectorAll('.tour-item');
tourItems.forEach((item) => {
  item.addEventListener('click', () => {
    tourItems.forEach((other) => {
      other.classList.toggle('is-active', other === item);
      other.setAttribute('aria-selected', String(other === item));
    });
    if (!tourShot) return;
    const src = `assets/images/${item.dataset.shot}.png`;
    if (tourShot.getAttribute('src') === src) return;
    tourShot.classList.add('is-swapping');
    const next = new Image();
    next.onload = () => {
      tourShot.src = src;
      tourShot.alt = item.dataset.alt || '';
      tourShot.classList.remove('is-swapping');
    };
    next.onerror = () => tourShot.classList.remove('is-swapping');
    next.src = src;
  });
});

// Platform filter: dims the cards that don't match.
const segButtons = document.querySelectorAll('.seg-btn');
const plans = document.querySelectorAll('.plan');
segButtons.forEach((btn) => {
  btn.addEventListener('click', () => {
    const filter = btn.dataset.filter;
    segButtons.forEach((b) => {
      b.classList.toggle('is-active', b === btn);
      b.setAttribute('aria-selected', String(b === btn));
    });
    plans.forEach((plan) => {
      plan.classList.toggle('is-dim', filter !== 'all' && plan.dataset.kind !== filter);
    });
  });
});

// Theme switch (remembered per browser when storage is available).
const root = document.documentElement;
const themeButtons = document.querySelectorAll('[data-theme-set]');
function currentTheme() {
  if (root.dataset.theme) return root.dataset.theme;
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}
function markTheme() {
  const theme = currentTheme();
  themeButtons.forEach((b) => b.classList.toggle('is-active', b.dataset.themeSet === theme));
}
themeButtons.forEach((btn) => {
  btn.addEventListener('click', () => {
    root.dataset.theme = btn.dataset.themeSet;
    try { localStorage.setItem('wpx-theme', btn.dataset.themeSet); } catch (e) { /* storage unavailable */ }
    markTheme();
  });
});
markTheme();

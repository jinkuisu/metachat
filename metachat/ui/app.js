// MetaChat UI Application
// Communicates with C++ engine via MessageBus

(function() {
  'use strict';

  const sidebar = document.getElementById('sidebar');
  const dashboard = document.getElementById('dashboard');
  const platformGrid = document.getElementById('platform-grid');
  let accounts = [];
  let activeAccountId = null;

  // ?? Platform icons (emoji as placeholder) ??
  const platformIcons = {
    whatsapp: '??', telegram: '??', facebook: '??',
    instagram: '??', twitter: '??', default: '??'
  };

  function getIcon(type) {
    return platformIcons[type] || platformIcons.default;
  }

  // ?? Render platform cards on dashboard ??
  function renderDashboard(list) {
    const byPlatform = {};
    list.forEach(a => {
      const p = a.type || 'default';
      if (!byPlatform[p]) byPlatform[p] = [];
      byPlatform[p].push(a);
    });

    platformGrid.innerHTML = '';
    for (const [platform, accs] of Object.entries(byPlatform)) {
      const card = document.createElement('div');
      card.className = 'platform-card';
      const unread = accs.reduce((s, a) => s + (a.unread_count || 0), 0);
      card.innerHTML = '<div class="icon">' + getIcon(platform) + '</div>' +
        '<div class="name">' + platform.charAt(0).toUpperCase() + platform.slice(1) + '</div>' +
        '<div class="count">' + accs.length + ' accounts' + (unread > 0 ? ' \u00b7 ' + unread + ' unread' : '') + '</div>';
      card.onclick = function() { toggleAccountList(platform); };
      platformGrid.appendChild(card);
    }
  }

  // ?? Toggle account list for a platform ??
  function toggleAccountList(platform) {
    const existing = document.getElementById('account-list');
    if (existing) existing.remove();

    const list = document.createElement('div');
    list.id = 'account-list';
    list.className = 'visible';

    const filtered = accounts.filter(function(a) { return (a.type || 'default') === platform; });
    filtered.forEach(function(a) {
      const item = document.createElement('div');
      item.className = 'account-item';
      if (a.id === activeAccountId) item.classList.add('active');
      item.textContent = a.name || a.id;
      item.onclick = function() { switchToAccount(a.id); };
      list.appendChild(item);
    });

    document.body.appendChild(list);

    document.addEventListener('click', function handler(e) {
      if (!list.contains(e.target)) {
        list.remove();
        document.removeEventListener('click', handler);
      }
    });
  }

  // ?? Switch to an account ??
  function switchToAccount(accountId) {
    const a = accounts.find(function(x) { return x.id === accountId; });
    if (!a) return;

    activeAccountId = accountId;

    // Update sidebar avatars
    document.querySelectorAll('#sidebar .avatar').forEach(function(el) {
      el.classList.toggle('active', el.dataset.accountId === accountId);
    });

    // Hide dashboard, show overlay
    dashboard.classList.remove('visible');

    // Remove account list if visible
    var al = document.getElementById('account-list');
    if (al) al.remove();

    // Send switch command to C++
    if (window.MessageBus) {
      MessageBus.send('webs', 'switch', {
        account_id: a.id,
        url: a.url || 'about:blank'
      });
    }
  }

  // ?? Go back to dashboard ??
  function goToDashboard() {
    activeAccountId = null;
    dashboard.classList.add('visible');

    document.querySelectorAll('#sidebar .avatar').forEach(function(el) {
      el.classList.remove('active');
    });

    if (window.MessageBus) {
      MessageBus.send('webs', 'show_dashboard', {});
    }
  }

  // ?? Render sidebar avatars ??
  function renderSidebar(list) {
    sidebar.innerHTML = '';
    // Home button
    const home = document.createElement('div');
    home.className = 'avatar';
    home.textContent = '\u2302';
    home.title = 'Dashboard';
    home.onclick = goToDashboard;
    sidebar.appendChild(home);

    list.forEach(function(a) {
      const el = document.createElement('div');
      el.className = 'avatar';
      el.dataset.accountId = a.id;
      el.textContent = (a.name || a.id).charAt(0).toUpperCase();
      el.title = a.name || a.id;
      if (a.id === activeAccountId) el.classList.add('active');
      el.onclick = function() { switchToAccount(a.id); };
      sidebar.appendChild(el);
    });
  }

  // ?? Initialize: load accounts from C++ ??
  function init() {
    if (window.MessageBus) {
      MessageBus.send('accounts', 'list').then(function(res) {
        accounts = (res.payload && res.payload.accounts) || [];
        renderSidebar(accounts);
        renderDashboard(accounts);
      }).catch(function(err) {
        console.error('MetaChat: failed to load accounts', err);
      });

      // Listen for events from C++
      MessageBus.on('accounts', 'updated', function(msg) {
        accounts = (msg.payload && msg.payload.accounts) || [];
        renderSidebar(accounts);
        renderDashboard(accounts);
      });
    } else {
      // Fallback: show static UI
      console.warn('MetaChat: MessageBus not available');
      dashboard.classList.add('visible');
    }
  }

  // ?? Start when DOM is ready ??
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();

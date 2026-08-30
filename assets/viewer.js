/* fy-docs viewer: theme menu, collapsible sidebar, per-chapter paging. */
(function () {
  'use strict';

  var THEME_KEY = 'fydocs-theme';
  var SIDEBAR_KEY = 'fydocs-sidebar';
  var SIDEBAR_WIDTH_KEY = 'fydocs-sidebar-width';
  var MOBILE_BREAKPOINT = 1024;
  var MIN_SIDEBAR_WIDTH = 192;
  var MAX_SIDEBAR_WIDTH = 512;
  var THEME_CLASSES = ['light', 'rust', 'coal', 'navy', 'ayu'];
  var CHINESE = document.documentElement.lang.toLowerCase().indexOf('zh') === 0;
  var TEXT = CHINESE ? {
    cover: '封面',
    noMatches: '没有匹配的章节',
    chapterNavigation: '章节导航',
    previous: '上一页',
    next: '下一页',
    position: function (current, total) { return '第 ' + current + ' / ' + total + ' 节'; }
  } : {
    cover: 'Cover',
    noMatches: 'No matching chapters',
    chapterNavigation: 'Chapter navigation',
    previous: 'Previous',
    next: 'Next',
    position: function (current, total) { return 'Chapter ' + current + ' of ' + total; }
  };

  function $(id) { return document.getElementById(id); }
  function readSetting(key) {
    try { return window.localStorage ? window.localStorage.getItem(key) : null; }
    catch (_) { return null; }
  }
  function writeSetting(key, value) {
    try { if (window.localStorage) window.localStorage.setItem(key, value); }
    catch (_) { /* Private or file:// pages may forbid persistent storage. */ }
  }

  /* ---------- theme palette menu ---------- */

  var themeToggle = $('fy-theme-toggle');
  var themeMenu = $('fy-theme-menu');

  function storedTheme() {
    var stored = readSetting(THEME_KEY);
    if (!stored || stored === 'dark') stored = 'preference';
    return stored;
  }

  function paintTheme() {
    var stored = storedTheme();
    var active = stored === 'preference'
      ? (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'navy' : 'light')
      : stored;
    var root = document.documentElement;
    THEME_CLASSES.forEach(function (name) { root.classList.remove(name); });
    root.classList.add(active);
    var items = themeMenu.querySelectorAll('[data-theme]');
    Array.prototype.forEach.call(items, function (item) {
      item.setAttribute('aria-checked', item.getAttribute('data-theme') === stored ? 'true' : 'false');
    });
  }

  function setMenu(open) {
    themeMenu.hidden = !open;
    themeToggle.setAttribute('aria-expanded', String(open));
  }

  themeToggle.addEventListener('click', function (event) {
    event.stopPropagation();
    setMenu(themeMenu.hidden);
  });
  document.addEventListener('click', function (event) {
    if (!themeMenu.hidden && !themeMenu.contains(event.target)) setMenu(false);
  });
  Array.prototype.forEach.call(themeMenu.querySelectorAll('[data-theme]'), function (item) {
    item.addEventListener('click', function () {
      writeSetting(THEME_KEY, item.getAttribute('data-theme'));
      paintTheme();
      setMenu(false);
    });
  });
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', paintTheme);
  paintTheme();

  /* ---------- collapsible sidebar ---------- */

  var columns = document.querySelector('.fy-columns');
  var sidebar = $('doc-sidebar');
  var sidebarToggle = $('fy-sidebar-toggle');
  var sidebarResize = $('fy-sidebar-resize');
  var scrim = $('fy-scrim');
  var shell = document.querySelector('.fy-shell');

  function maximumSidebarWidth() {
    return Math.min(MAX_SIDEBAR_WIDTH, Math.floor(window.innerWidth * 0.55));
  }

  function sidebarWidth(value) {
    var width = Math.max(MIN_SIDEBAR_WIDTH, Math.min(maximumSidebarWidth(), Math.round(value)));
    document.documentElement.style.setProperty('--fy-sidebar-width', width + 'px');
    return width;
  }

  var savedWidth = Number(readSetting(SIDEBAR_WIDTH_KEY));
  if (Number.isFinite(savedWidth) && savedWidth > 0) sidebarWidth(savedWidth);

  function setSidebar(open) {
    columns.classList.toggle('fy-sidebar-open', open);
    shell.classList.toggle('fy-sidebar-open', open);
    sidebarToggle.setAttribute('aria-expanded', String(open));
    writeSetting(SIDEBAR_KEY, open ? '1' : '0');
  }

  var savedSidebar = readSetting(SIDEBAR_KEY);
  setSidebar(savedSidebar === null
    ? window.innerWidth >= MOBILE_BREAKPOINT
    : savedSidebar === '1');

  sidebarToggle.addEventListener('click', function () {
    setSidebar(!columns.classList.contains('fy-sidebar-open'));
  });
  scrim.addEventListener('click', function () { setSidebar(false); });
  sidebarResize.addEventListener('pointerdown', function (event) {
    if (window.innerWidth < MOBILE_BREAKPOINT) return;
    event.preventDefault();
    shell.classList.add('fy-sidebar-resizing');
    sidebarResize.setPointerCapture(event.pointerId);
  });
  sidebarResize.addEventListener('pointermove', function (event) {
    if (!shell.classList.contains('fy-sidebar-resizing')) return;
    sidebarWidth(event.clientX);
  });
  sidebarResize.addEventListener('pointerup', function (event) {
    if (!shell.classList.contains('fy-sidebar-resizing')) return;
    shell.classList.remove('fy-sidebar-resizing');
    writeSetting(SIDEBAR_WIDTH_KEY, String(sidebarWidth(event.clientX)));
  });
  sidebarResize.addEventListener('lostpointercapture', function () {
    shell.classList.remove('fy-sidebar-resizing');
  });
  window.addEventListener('resize', function () {
    sidebarWidth(sidebar.getBoundingClientRect().width);
  });

  /* ---------- per-chapter paging ---------- */

  var body = $('doc-body');
  var toc = body.querySelector('nav[role="doc-toc"]');
  if (toc) sidebar.appendChild(toc);

  var chapters = [];
  var idToChapter = Object.create(null);

  if (toc) {
    var group = null;
    Array.prototype.slice.call(body.children).forEach(function (node) {
      if (node === toc) return;
      if (node.tagName === 'H2') {
        group = { title: node.textContent.trim(), el: document.createElement('section') };
        group.el.className = 'fy-chapter';
        group.nodes = [node];
        body.appendChild(group.el);
        chapters.push(group);
      } else if (group) {
        group.nodes.push(node);
      } else {
        group = { title: TEXT.cover, el: document.createElement('section') };
        group.el.className = 'fy-chapter';
        group.nodes = [node];
        body.appendChild(group.el);
        chapters.push(group);
      }
    });
    chapters.forEach(function (chapter, idx) {
      chapter.nodes.forEach(function (node) { chapter.el.appendChild(node); });
      if (!chapter.nodes[0].id) {
        chapter.nodes[0].id = idx === 0 ? 'cover' : 'ch-' + idx;
      }
      chapter.firstId = chapter.nodes[0].id;
      Array.prototype.forEach.call(chapter.el.querySelectorAll('[id]'), function (node) {
        idToChapter[node.id] = idx;
      });
      idToChapter[chapter.firstId] = idx;
    });
  }

  /* ---------- document search and printing ---------- */

  var searchToggle = $('fy-search-toggle');
  var searchPanel = $('fy-search-panel');
  var searchInput = $('fy-search-input');
  var searchResults = $('fy-search-results');

  function setSearch(open) {
    searchPanel.hidden = !open;
    searchToggle.setAttribute('aria-expanded', String(open));
    if (open) window.setTimeout(function () { searchInput.focus(); }, 0);
  }

  function search(query) {
    searchResults.textContent = '';
    query = query.trim().toLocaleLowerCase();
    if (!query) return;
    var matches = chapters.filter(function (chapter) {
      return chapter.el.textContent.toLocaleLowerCase().indexOf(query) !== -1;
    });
    if (!matches.length) {
      var empty = document.createElement('p');
      empty.className = 'fy-search-empty';
      empty.textContent = TEXT.noMatches;
      searchResults.appendChild(empty);
      return;
    }
    matches.forEach(function (chapter) {
      var index = chapters.indexOf(chapter);
      var result = document.createElement('button');
      result.className = 'fy-search-result';
      result.type = 'button';
      result.textContent = chapter.title;
      result.addEventListener('click', function () {
        setSearch(false);
        go(index, chapter.firstId);
      });
      searchResults.appendChild(result);
    });
  }

  searchToggle.addEventListener('click', function (event) {
    event.stopPropagation();
    setSearch(searchPanel.hidden);
  });
  searchInput.addEventListener('input', function () { search(searchInput.value); });
  $('fy-print').addEventListener('click', function () { window.print(); });
  document.addEventListener('click', function (event) {
    if (!searchPanel.hidden && !searchPanel.contains(event.target) && event.target !== searchToggle) {
      setSearch(false);
    }
  });

  var pager = null;
  var position = null;
  var prevLink = null;
  var nextLink = null;
  var active = -1;

  function makePager() {
    pager = document.createElement('nav');
    pager.className = 'fy-pager';
    pager.setAttribute('aria-label', TEXT.chapterNavigation);

    function makeLink(label, extraClass) {
      var link = document.createElement('a');
      link.className = 'fy-pager-link ' + extraClass;
      var small = document.createElement('span');
      small.className = 'fy-pager-label';
      small.textContent = label;
      var title = document.createElement('span');
      title.className = 'fy-pager-title';
      link.appendChild(small);
      link.appendChild(title);
      return link;
    }

    prevLink = makeLink(TEXT.previous, 'fy-pager-prev');
    nextLink = makeLink(TEXT.next, 'fy-pager-next');
    position = document.createElement('span');
    position.className = 'fy-pager-pos';
    pager.appendChild(prevLink);
    pager.appendChild(position);
    pager.appendChild(nextLink);
    document.querySelector('.fy-page').appendChild(pager);
  }

  function fillLink(link, index) {
    var hidden = index < 0 || index >= chapters.length;
    link.classList.toggle('fy-pager-hidden', hidden);
    link.href = hidden ? '#' : '#' + (chapters[index].firstId || '');
    var title = link.querySelector('.fy-pager-title');
    title.textContent = hidden ? '' : chapters[index].title;
    link.setAttribute('aria-hidden', String(hidden));
    link.tabIndex = hidden ? -1 : 0;
  }

  function go(index, anchorId) {
    if (index < 0 || index >= chapters.length) return;
    active = index;
    chapters.forEach(function (chapter, i) {
      chapter.el.classList.toggle('fy-chapter-active', i === index);
    });
    fillLink(prevLink, index - 1);
    fillLink(nextLink, index + 1);
    position.textContent = TEXT.position(index + 1, chapters.length);

    Array.prototype.forEach.call(toc.querySelectorAll('.fy-toc-active'), function (node) {
      node.classList.remove('fy-toc-active');
    });
    var firstId = chapters[index].firstId;
    if (firstId) {
      var tocLink = toc.querySelector('a[href="#' + firstId + '"]');
      if (tocLink) tocLink.classList.add('fy-toc-active');
    }

    if (window.innerWidth < MOBILE_BREAKPOINT) setSidebar(false);

    if (anchorId) {
      var target = document.getElementById(anchorId);
      if (target) {
        target.scrollIntoView({ block: 'start' });
        history.replaceState(null, '', '#' + anchorId);
        return;
      }
    }
    history.replaceState(null, '', window.location.pathname + window.location.search);
    document.querySelector('.fy-content').scrollTop = 0;
  }

  document.addEventListener('click', function (event) {
    var link = event.target.closest ? event.target.closest('a[href^="#"]') : null;
    if (!link) return;
    var id = decodeURIComponent(link.getAttribute('href').slice(1));
    if (!Object.prototype.hasOwnProperty.call(idToChapter, id)) return;
    event.preventDefault();
    go(idToChapter[id], id);
  });

  window.addEventListener('hashchange', function () {
    var id = decodeURIComponent(window.location.hash.slice(1));
    if (Object.prototype.hasOwnProperty.call(idToChapter, id)) go(idToChapter[id], id);
  });

  document.addEventListener('keydown', function (event) {
    if (event.defaultPrevented || event.ctrlKey || event.metaKey || event.altKey) return;
    if (event.key !== 'ArrowRight' && event.key !== 'ArrowLeft') return;
    var tag = (event.target.tagName || '').toLowerCase();
    if (tag === 'input' || tag === 'textarea' || tag === 'select' || event.target.isContentEditable) return;
    go(active + (event.key === 'ArrowRight' ? 1 : -1));
  });

  var toolbarTitle = document.querySelector('.fy-toolbar-title');
  var sidebarBrand = document.querySelector('.fy-sidebar-brand');

  if (toolbarTitle) {
    toolbarTitle.title = TEXT.cover;
    toolbarTitle.addEventListener('click', function () {
      go(0, 'cover');
    });
  }
  if (sidebarBrand) {
    sidebarBrand.title = TEXT.cover;
    sidebarBrand.addEventListener('click', function () {
      go(0, 'cover');
    });
  }

  if (chapters.length > 1) {
    makePager();
    var startId = decodeURIComponent(window.location.hash.slice(1));
    var start = Object.prototype.hasOwnProperty.call(idToChapter, startId) ? idToChapter[startId] : 0;
    go(start, start || null);
  }
})();


// fy-docs live reload: reload the page when the served build id changes.
(function poll() {
  if (document.visibilityState === 'hidden') {
    return setTimeout(poll, 1500);
  }
  fetch('_build', { cache: 'no-store' })
    .then(function (response) { return response.text(); })
    .then(function (id) {
      if (window.__fyBuild === undefined) {
        window.__fyBuild = id;
      } else if (window.__fyBuild !== id) {
        return location.reload();
      }
      setTimeout(poll, 1500);
    })
    .catch(function () { setTimeout(poll, 3000); });
})();

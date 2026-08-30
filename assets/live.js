// fy-docs live reload: subscribe to the dev server's event stream.
// Static builds have no server; the failed connection closes silently.
(function () {
  if (!window.EventSource) return;
  var source = new EventSource('events');
  source.onmessage = function (event) {
    if (window.__fyBuild === undefined) {
      window.__fyBuild = event.data;
    } else if (window.__fyBuild !== event.data) {
      location.reload();
    }
  };
  source.onerror = function () {
    source.close();
  };
})();

#import "../../fy-spec/lib.typ": *

= server Module: Dev Server & Live Reload <sec-server>

The `server` module runs exclusively under `cargo fy-docs dev`, providing an Axum-powered development server.

#contract[
  `server` binds to `127.0.0.1:8181` (auto-incrementing upon port collision) and exposes `_poll.js` for seamless live reloading upon source edits.
]

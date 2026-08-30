#import "../../fy-spec/lib.typ": *

= server Module: Dev Server & Live Reload <sec-server>

The `server` module runs exclusively under `cargo fy-docs dev`, providing an Axum-powered development server.

#contract[
  `server` binds to `127.0.0.1:8181` (auto-incrementing upon port collision) and exposes a `/events` Server-Sent-Events stream that pushes a new build id after every rebuild; open pages reload themselves the moment the id changes. The same `live.js` client ships with static builds, where the failed connection closes silently.
]

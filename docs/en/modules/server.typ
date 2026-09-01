#import "../../fy-spec/lib.typ": *

= server Module: Dev Server, Watcher & Shared State <sec-server>

This chapter specifies `src/server.rs` together with the two support modules folded into it: `src/watcher.rs`, which turns source edits into rebuilds, and `src/state.rs`, the state object a dev session shares between its threads. The HTTP surface exists only under `cargo fy-docs dev`; the state type is on the path of every compiling command, because `compiler::generate` reads its captured options from there.

#contract[
  The router has exactly one route: `GET /events`, an SSE stream. Every other request is answered by a static file read out of `docs/target/`, and a path with no generated file is a plain not-found. No request ever triggers a compilation; the server only exposes what a generation already wrote.
]

#contract[
  `dev` starts in a fixed order: first generation, watcher spawn, router construction, port bind, optional browser open, then serve. The listening socket is opened only after the first generation returns, so a slow first compile never leaves a port accepting requests for pages that do not exist yet. A first generation that fails still starts the server — the browser is shown the error page it just wrote.
]

#contract[
  The build id is a monotonic counter starting at `1`. One broadcast channel carries it; each `/events` subscription gets a receiver of its own, and every subscriber sees the same values. The stream is infallible and keeps a default keep-alive so an idle page is still a live page.
]

#contract[
  A newly subscribed page receives the current build id as its first frame, and that frame is its baseline; each later bump is one data frame carrying the new id. Emitting the baseline is what makes the first save after a page opens reload it: a stream that reported only changes would leave every page permanently one rebuild behind.
]

#contract[
  The build id is bumped after every generation attempt, whether it succeeded or failed. A failed rebuild must still push a new id, or the open page would keep showing the last good build and never learn that the source broke.
]

#invariant[
  The state holds one receiver for the whole process lifetime solely to keep the broadcast channel open. A channel with no receiver is closed, and every later update would be dropped without an error — the anchor field exists because the alternative is a dev server that stops reloading in silence.
]

#contract[
  The watcher is recursive over the `docs/` directory plus the top-level directory of every local import the project resolves, deduplicated. A `.typ` file change is a source change; a change under `target/` or `release/` is not, even if a `.typ`-suffixed artifact appears there, because the watcher must never react to its own output. Nothing outside those directories can trigger a rebuild.
]

#contract[
  A burst of changes folds into one rebuild: the watcher keeps draining while changes arrive, but the fold is bounded by a quiet period of 500 ms that slides on each change and a hard cap of 2 s measured from the first change. Only the quiet period would let a process writing sources continuously postpone the rebuild forever.
]

#contract[
  A rebuild that fails is discarded and the watch loop returns to waiting; the loop ends only when the notification channel closes. The dev session survives on error pages, not on process liveness. Panics become failures only inside the parallel compile step, which collects them (see the `compiler` chapter); the watch thread carries no panic barrier of its own, so a panic elsewhere in a rebuild stops that session from rebuilding again.
]

#contract[
  `ctrl-c` stops the server through the graceful-shutdown path, logs a shutdown line, and lets the process exit; the watcher thread is a plain thread that ends with the process and is never joined.
]

#contract[
  The same `live.js` client ships with a one-time static build, where connecting to `/events` fails and the failure is swallowed: a page opened from `file://` must not surface a reload error it cannot use.
]

== Module Structure

#logic-box[
  `server::router(state: &Arc<AppState>) -> Router` builds the two-arm surface and takes a
  subscription handle at construction time.
  `watcher::spawn(state: Arc<AppState>) -> Result<()>` registers the watches and returns once the
  background thread is running; a directory it cannot watch is an error to `dev`, raised before any
  port is bound.
]

#logic-box[
  `AppState { project: Project, build_id: AtomicU64, generate: GenerateOptions, events:
  watch::Sender<u64>, _events_anchor: watch::Receiver<u64> }` — the sender and the anchor are
  private; the only way in is `AppState::with_generate(project, generate)`, `subscribe()`,
  `current_build_id()` and `bump_build()`. `GenerateOptions { with_pdf: bool, lang_filter:
  Option<String> }` is the captured startup options, and the counter and the broadcast value are
  kept in step only by `bump_build`, which is what makes the served id and the logged build number
  agree.
]

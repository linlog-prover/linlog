# Web client stack: research, 2026-10-18

Every source was read on 2026-10-18; dates in the text are release or
publication dates. Version numbers come from the npm registry, crates.io and
GitHub releases pages read the same day. "Not verified" marks what no source
settled. Nothing was installed and no npm command was run.

## Summary of recommendations

- **Stack**: Astro 7.4 + TypeScript + Solid **1.9.x**, the app as one
  `client:only="solid-js"` island, the written content as Markdown/MDX content
  collections. This is the author's expectation, with one correction: Solid
  2.0 is still a release candidate and `@astrojs/solid-js` 7.1.0 accepts
  `solid-js ^1.9.13` only. Fallback if Astro gets in the way: plain Vite 8 +
  Solid (section 1).
- **Worker**: compile with `wasm-bindgen` (the CLI, nixpkgs has it per
  version), not `wasm-pack`; `wasm-opt` after it; one module worker created
  with `new Worker(new URL(..., import.meta.url), { type: "module" })`; a
  hand-rolled request-id protocol, not Comlink; cancel with `terminate()` and
  a fresh worker. GitHub Pages cannot send COOP/COEP (confirmed, including by a
  live header check), so no `SharedArrayBuffer`. Stack: 1 MiB by default,
  raised with `-C link-arg=-zstack-size=N`. Plan for about 256 MiB on iOS
  (section 2).
- **State in the URL**: fragment `#v1.<base64url(deflate-raw)>`, via
  `CompressionStream("deflate-raw")` (every browser since 2023), written with
  a debounced `history.replaceState`, kept under about 2000 characters
  (section 3).
- **Offline**: not in the first version. Later: `vite-plugin-pwa` 2.0.0, with
  a navigation denylist for `/linlog/`, because all Pages sites of the
  organization share one origin. `@vite-pwa/astro` 1.2.0 lists Astro up to 5
  only (section 4).
- **Build/CI**: Nix `fetchPnpmDeps` (`fetcherVersion = 4`) + `pnpmConfigHook`
  + `pnpmBuildHook` with `pnpm_11`, Node 24; deploy with
  `upload-pages-artifact@v5` and `deploy-pages@v5`. Repository
  `linlog-prover.github.io` serves the organization root; `/linlog` stays the
  rustdoc project site. No `.nojekyll`; a `404.html` is wanted (section 5).
- **Licences**: Vite 8 has a built-in `build.license` option that writes the
  bundled npm licences to a file; the Rust crates inside the wasm need their own
  notice (`cargo-about`). Euler Math 0.75 is OFL 1.1 with **no Reserved Font
  Name**; subsetting counts as modification (FAQ 2.6) but without an RFN that
  needs no rename; keep the licence records (`pyftsubset --name-IDs='*'`, the
  default drops them) and ship `OFL.txt` beside it (section 6).
- **Quality**: WCAG 2.2 AA; Focus Appearance (2.4.13) is AAA, not AA. Never put
  interactive elements inside `role="img"`. Budget scale: 170 KB compressed
  critical path (web.dev, slow 3G); load the wasm after first paint. Say in the
  footer that GitHub logs visitor IPs (section 7).
- **Precedents**: the Lean 4 web editor is not one (its server runs on a
  host). Pyodide, Excalidraw, egui's eframe template are (section 8).

## 1. Stack

| Item | Version, date | Source |
|---|---|---|
| Astro | 7.4.0, 2026-10-15 (7.0: 2026-06-22) | https://astro.build/blog/ |
| `@astrojs/solid-js` | 7.1.0, 2026-10-15; peer `solid-js ^1.9.13` | npm registry; https://docs.astro.build/en/guides/integrations-guide/solid-js/ |
| solid-js | latest 1.9.17 (2026-10-07); next 2.0.0-rc.15 (2026-10-13) | https://github.com/solidjs/solid/releases |
| vite-plugin-solid | 2.11.14 (Vite up to 9) | npm registry |
| Vite | 8.3.4 (2026-10-08); 8.0 on 2026-03-12, Rolldown | https://vite.dev/blog/announcing-vite8 |
| Node | Astro 7 needs >= 22.22.0; `withastro/action` defaults to 24 | https://github.com/withastro/astro/releases |

- Astro 7 (https://astro.build/blog/astro-7/, upgrade guide
  https://docs.astro.build/en/guides/upgrade-to/v7/): Vite 8, Rust compiler
  (unclosed tags are errors), Rust Markdown pipeline, `compressHTML: 'jsx'`
  default, `@astrojs/db` removed. Static output is still the default and
  the build goes to `dist/` with assets in `_astro/`
  (https://docs.astro.build/en/reference/configuration-reference/).
- Islands: no JS ships without a `client:*` directive; `client:only="solid-js"`
  skips server rendering, which suits an app whose logic is wasm in a worker
  (https://docs.astro.build/en/reference/directives-reference/).
- Content collections: core since 2.0, `glob()` loader, Zod 4 schemas, MDX
  for build-time collections
  (https://docs.astro.build/en/guides/content-collections/).
- Solid integration needs `"jsx": "preserve"` and `"jsxImportSource":
  "solid-js"`. Solid 2.0 is pre-release on the project's releases page (RC
  announcement: async in the core, Solid 3.0 retired, SolidStart in
  maintenance). Use 1.9; moving to 2.0 is its own step later.
- Astro's docs say nothing on workers
  (https://docs.astro.build/en/guides/client-side-scripts/); they say WASM
  files load "using the browser's WebAssembly API" and Vite's `?url`/`?raw`
  imports work (https://docs.astro.build/en/guides/imports/). Not verified: a
  documented `new Worker(new URL(...))` inside an Astro island. Test in the
  first scaffold, development and production build.

Alternatives (sizes: bundlephobia on the whole package, 2026-10-18, so
overstated for a tree-shaken hello world; blog comparisons disagree, so only
the order of magnitude counts):

| Option | Runtime (min+gz) | Status | Worker/wasm | Reads for a Rust author |
|---|---|---|---|---|
| Solid 1.9 | 8.4 KB | 1.9.17, 2.0 RC | same for all JS options: the worker is plain TS | signals, JSX |
| Svelte 5 | 10.8 KB (main entry) | 5.57.2 | same | compiled components, runes |
| Preact 11 | 5.0 KB (site: "3kB") | 11.0.1 | same | React model, most documentation |
| Lit 3 | 6.0 KB | 3.3.3 | same | classes, shadow DOM hinders styling SVG |
| Leptos | wasm only, no figure in docs | 0.8.22, 0.9.0-rc0 (2026-10-15) | no worker guidance (web-sys) | Rust, signals, `view!` |
| Dioxus | wasm only | 0.7.10 (2026-07-30), 0.8.0-alpha.3 | none found | Rust, React-like |
| Yew | wasm only | 0.23.0 (2026-03); `yew-agent` 0.5.0 | `yew-agent` for workers | Rust, React-like |

Sources: bundlephobia API; the projects' GitHub releases pages;
https://book.leptos.dev/deployment/binary_size.html (wasm binaries are
"significantly larger" than the JS equivalent; no figures).

- A Rust UI puts a second wasm module (or the whole prover) on the main thread,
  while the prover must run in a worker. A UI crate depending on `linlog` also
  makes the two-repository split (D22) harder, and needs trunk or dx pinned
  in the flake. Hello-world sizes of the Rust frameworks: not verified.
- The JS options differ by a few KB; the prover's wasm and the SVG drawings
  dwarf them. Size does not decide; maintenance does not either (all are
  active). Readability and the written-content plan do.
- Recommendation: Astro + Solid 1.9. The plan has explanations, tutorials and
  exercises under CC BY, which is Astro's use case. Svelte 5 would be as
  sound if the author preferred it.

## 2. The wasm boundary

### wasm-bindgen, wasm-pack, hand-written ABI

- wasm-bindgen 0.2.129 (2026-09-25) lives in its own organization since the
  `rustwasm` org was sunset
  (https://blog.rust-lang.org/inside-rust/2025/07/21/sunsetting-the-rustwasm-github-org/ :
  wasm-bindgen moves with new maintainers; wasm-pack was to be archived
  unless a maintainer took it). wasm-pack continues at
  https://github.com/wasm-bindgen/wasm-pack (v0.15.0, 15 May, README names
  `drager`). Not verified: how active. Call the `wasm-bindgen` CLI
  directly; wasm-pack only wraps it and `wasm-opt`.
- nixpkgs has `wasm-bindgen-cli_0_2_N` for many versions and
  `buildWasmBindgenCli` (https://github.com/NixOS/nixpkgs/tree/master/pkgs/by-name/wa).
  The CLI must match the crate version in `Cargo.lock`
  (https://wasm-bindgen.github.io/wasm-bindgen/wasm-bindgen-test/usage.html),
  so the flake picks the attribute from the lock.
- Targets (https://wasm-bindgen.github.io/wasm-bindgen/reference/deployment.html):
  `bundler` (default), `web` (ES module, loads without post-processing; `init`
  takes a URL, `Response`, `ArrayBuffer` or `WebAssembly.Module`,
  https://wasm-bindgen.github.io/wasm-bindgen/examples/without-a-bundler.html),
  `no-modules`, `nodejs`, `deno`, `module` (source-phase imports, esbuild and
  Node 24 only). For a Vite worker the least magic is `--target web` with
  `init({ module_or_path: wasmUrl })` and `import wasmUrl from "x_bg.wasm?url"`.
  The official worker example still uses `no-modules` because it dates from
  when only Chrome had module workers
  (https://wasm-bindgen.github.io/wasm-bindgen/examples/wasm-in-web-worker.html).
- Strings cross by copy through `TextEncoder`/`TextDecoder`
  (https://wasm-bindgen.github.io/wasm-bindgen/reference/types/string.html):
  one copy each way per call, small beside a search. A hand-written ABI
  (`alloc`/`dealloc`, pointer+length) saves a few KB of glue (not measured)
  but loses `wasm-bindgen-test` (Node by default, browsers via WebDriver) and
  rewrites string and panic handling. Recommendation: wasm-bindgen with a few
  `String -> String` functions, so a hand ABI stays possible.

### Size and hosting

- Release profile `opt-level = 's'|'z'` (measure both), `lto = true`, then
  `wasm-opt -Os|-Oz` for "another 15-20%"
  (https://rustwasm.github.io/docs/book/reference/code-size.html, an archived
  book); Leptos' book adds `codegen-units = 1`. Binaryen version_133 is from
  2026-09-21 (https://github.com/WebAssembly/binaryen/releases). Measure
  `*_bg.wasm`, not cargo's output
  (https://wasm-bindgen.github.io/wasm-bindgen/reference/optimize-size.html).
- Live check of Pages on 2026-10-18 (`curl -I` of an egui demo's wasm,
  emilk.github.io/eframe_template): `content-type: application/wasm`,
  `content-encoding: gzip` (7.7 MB to 3.1 MB; no Brotli even when requested),
  `cache-control: max-age=600`. So budgets are in gzip bytes and precompressed
  `.br` files cannot be served.

### Loading in a worker from Vite/Astro

- Vite recommends `new Worker(new URL('./w.ts', import.meta.url), { type:
  'module' })` with static options; `?worker` is the alternative
  (https://vite.dev/guide/features.html#web-workers).
- Module workers: Chrome 80, Firefox 114, Safari 15, 97% of usage
  (https://caniuse.com/mdn-api_worker_worker_ecmascript_modules). Vite 8's
  default `build.target` is chrome111, edge111, firefox114, safari16.4,
  ios16.4 (https://vite.dev/config/build-options), so module workers are
  inside it. Old advice to avoid ES workers for Firefox (vite-plugin-wasm's
  README) predates Firefox 114.
- `worker.format` defaults to `'iife'`; `worker.rollupOptions` is a deprecated
  alias of `worker.rolldownOptions`
  (https://vite.dev/config/worker-options). Not verified: whether a worker with
  top-level await or wasm glue needs `'es'`; set `worker: { format: "es" }`
  and check the production build.
- Vite 8 imports `.wasm` natively (ESM integration needs top-level await;
  `?init` returns an instantiate function; `?url` plus
  `WebAssembly.instantiateStreaming` to reuse the `Module`)
  (https://vite.dev/guide/features.html#webassembly). `vite-plugin-wasm` 3.6.0
  is only needed for wasm-bindgen's `bundler` target.

### Messaging and cancelling

- Comlink 4.4.2 (2024-11-07, Apache-2.0, 1.2 KB gz) is stable, quiet for 23
  months, and gives RPC proxies without cancel or progress
  (https://github.com/GoogleChromeLabs/comlink). A protocol of `{id, kind,
  payload}` in and `{id, kind: result|progress|error}` out is about 40 lines
  and states what a cancel does. Pyodide's worker guide uses the same
  request-id design (https://pyodide.org/en/stable/usage/webworker.html).
- `Worker.terminate()` stops "at once", with no chance to finish
  (https://developer.mozilla.org/en-US/docs/Web/API/Worker/terminate). A restart
  needs a new instance: post the compiled `WebAssembly.Module` from the main
  thread (or rely on the HTTP cache) and re-send the state.
- A cooperative flag through `SharedArrayBuffer` needs cross-origin isolation:
  `Cross-Origin-Opener-Policy: same-origin` and `Cross-Origin-Embedder-Policy:
  require-corp|credentialless`
  (https://developer.mozilla.org/en-US/docs/Web/API/Window/crossOriginIsolated).
  **Confirmed: GitHub Pages cannot set headers.** The community thread
  https://github.com/orgs/community/discussions/13309 has a GitHub staff reply
  of July 2023 ("a scenario we would support with custom headers"), no
  timeline, users still asking in 2026. A header check of
  `https://linlog-prover.github.io/linlog/` shows `server: GitHub.com`,
  `cache-control: max-age=600`, `access-control-allow-origin: *`, no COOP or
  COEP. `coi-serviceworker` (https://github.com/gzuidhof/coi-serviceworker)
  works around it with a reload on first visit and a separate same-origin
  file; users report mixed results; not recommended. Pyodide's interrupts need
  the same headers and it names no fallback
  (https://pyodide.org/en/stable/usage/keyboard-interrupts.html).
- So: the engine's work-counting budget is the soft stop; `terminate()` the
  hard one, triggered from the main thread (a Stop button, or a `setTimeout`
  there; the main thread does no work, so the timer is reliable).

### Stack and memory

- rustc's wasm base target links with `stack-size=1048576` (1 MiB); LLD alone
  gives 64 KiB; the stack is placed before static data so an overflow traps
  (https://rust.googlesource.com/rust/+/HEAD/compiler/rustc_target/src/spec/base/wasm.rs).
  Override with `-C link-arg=-zstack-size=<bytes>` (a `build.rs`
  `cargo:rustc-link-arg=-zstack-size=0x800000`, or `RUSTFLAGS`). The rustc
  target page does not document the flag
  (https://doc.rust-lang.org/rustc/platform-support/wasm32-unknown-unknown.html;
  it does say `std::thread::spawn` panics and lists the default features:
  multivalue, mutable-globals, reference-types, sign-ext, and since Rust 1.87
  nontrapping-fptoint and bulk-memory). The flag rests on real build scripts,
  so the planned test at the limit also proves it took effect.
- wasm32 addresses at most 4 GiB; Chrome allows 4 GiB since M83
  (https://v8.dev/blog/4gb-wasm-memory). Tabs get less: WebKit bug 269777
  (2024-02, iOS 17) reports about 300 MB reliably usable by a wasm page against
  about 4 GB on Android Chrome, shrinking with reloads; closed as a duplicate
  of the "MegaBug" 269937 (https://bugs.webkit.org/show_bug.cgi?id=269777).
  A search summary of the thread says a fix was believed to ship in iOS 18;
  not read directly. Bug 221530: Safari reloaded the tab on out-of-memory
  instead of failing `memory.grow` (search summary only). Not verified:
  current iOS. A default near 256 MiB with refusal above it is an inference
  from these numbers; the library's refusal is the right failure mode, since
  a crash costs the page's state (section 3 puts it in the URL).

## 3. State and URLs

- The fragment is not sent in the request (URL standard; MDN's
  https://developer.mozilla.org/en-US/docs/Web/API/URL/hash does not state it).
  Base64url needs no percent-encoding; the fragment encode set is C0, space,
  `"`, `<`, `>` and backtick.
- `history.replaceState(state, "", url)` changes the URL without loading; it
  throws `SecurityError` "if called too frequently"
  (https://developer.mozilla.org/en-US/docs/Web/API/History/replaceState) and
  does not fire `hashchange`
  (https://developer.mozilla.org/en-US/docs/Web/API/Window/hashchange_event).
  Not verified: the Safari limit of 100 calls per 30 s. Debounce (about
  300 ms), try/catch, and `pushState` only for steps Back should undo.
- `CompressionStream` takes `gzip`, `deflate`, `deflate-raw`, works in workers,
  and is available in all browsers since May 2023
  (https://developer.mozilla.org/en-US/docs/Web/API/CompressionStream); "widely
  available" since 2025-11-09
  (https://web-platform-dx.github.io/web-features-explorer/features/compression-streams/).
  `deflate-raw` saves the zlib header and checksum. `lz-string` 1.5.0 was last
  published 2023-03-04 (MIT); no longer needed. Not verified:
  `Uint8Array.toBase64` (base64url) Baseline status, the MDN page 404ed; write
  the few lines with `btoa` and character replacement.
- Length: Chrome about 2 MB hard cap, Firefox about 65 000 characters, Safari
  about 80 000, with problems from about 2000 in practice and lower limits in
  messengers and mail (https://nuqs.dev/docs/limits; search summaries). No
  source measures fragments separately; servers never see them. Store the
  sequent text and the applied rule choices, not the derivation; replay them
  through the library on load, which also validates them. Above about 2000
  characters offer "download as file".
- A version prefix (`#v1.`) keeps old links decodable; an unknown version
  gets a message, as the plan does for `wire::LEVEL`. A link pasted into a
  chat reaches that chat's servers; GitHub never gets the fragment.

## 4. Offline

- A service worker's scope defaults to its script's directory, one worker per
  scope, and `skipWaiting()` can let a new worker serve pages the old one
  loaded (https://web.dev/articles/service-worker-lifecycle ,
  https://web.dev/learn/pwa/service-workers).
- **All Pages sites of one organization share one origin** (project sites
  differ only by path, GitHub's site-type table). A worker at the root would
  also control `/linlog/`, the rustdoc. Use a `navigateFallback` denylist for
  `^/linlog/` and other project paths, and prefix `localStorage` keys. This is
  inferred from the same-origin rule; GitHub's docs do not state it.
- `vite-plugin-pwa` 2.0.0 (2026-10-03, MIT; Vite 3 to 8, Workbox ^7.4.1);
  `workbox-build` 7.4.1 (2026-05-04). `@vite-pwa/astro` 1.2.0 (2025-11-27)
  declares Astro `^1..^5` and `vite-plugin-pwa ^1.2.0`: not Astro 7, not 2.0.
  Its docs (https://vite-pwa-org.netlify.app/frameworks/astro) say Astro
  components do not inject the PWA scripts; import `virtual:pwa-register`
  yourself. Not verified: it working under Astro 7. Alternative: the plugin in
  Astro's `vite.plugins`, or a hand-written worker file.
- Hand-written is realistic: egui's `eframe_template` has a short `sw.js` that
  precaches the page, JS and wasm on `install` and answers cache-first, with no
  update path (https://github.com/emilk/eframe_template). Excalidraw uses
  `VitePWA` with `registerType: "autoUpdate"`, fonts cached `CacheFirst` for
  90 days, `maximumFileSizeToCacheInBytes` 2.3 MB
  (https://github.com/excalidraw/excalidraw `excalidraw-app/vite.config.mts`).
- Worth it? Pages sends `max-age=600`; hashed `_astro/*` names make revalidation
  a cheap conditional request, so repeat visits are fast without a worker.
  Offline matters for a lecture hall with poor Wi-Fi, and the wasm is the one big
  asset. Costs: update flow, stale-cache hazard, the origin sharing, tests.
  Recommendation: not in version 1; later as its own step, `generateSW`,
  `registerType: "prompt"`, precache shell, wasm and font only, and a
  "ready offline" status message (4.1.3).

## 5. Build, CI, hosting

- nixpkgs manual, JavaScript chapter (raw `javascript.section.md` on master):
  `fetchPnpmDeps` (fixed-output; `fetcherVersion = 4` "for new packages";
  versions 1 and 2 removed in 26.11), `pnpmConfigHook`, `pnpmBuildHook` (runs
  `pnpm run build`), `pnpm` pinned to a major (`pnpm_11`; "regular breaking
  changes in the store format"). `pnpm.fetchDeps` is not in the chapter any
  more. `buildNpmPackage` (`npmDepsHash`, `importNpmLock`) is the npm route.
  pnpm's own latest is 12.14.0 (npm), nixpkgs has up to 11. Recommendation:
  pnpm 11 via `fetchPnpmDeps`: hash-pinned, offline in the sandbox; pnpm blocks
  install scripts unless approved and has `minimumReleaseAge` (https://pnpm.io/).
  Nix only for the toolchain would give up the hermetic build.
- The wasm module comes from this repository's flake input: the client's flake
  builds `linlog-web` with `wasm-bindgen-cli_0_2_N` and `binaryen`, and the pnpm
  derivation copies the output to `src/wasm/` in `preBuild`. Not verified: a
  worked pattern for that hand-over.
- Node 24 LTS (https://nodejs.org/en/about/previous-releases).
- Deployment (https://docs.astro.build/en/guides/deploy/github/ ,
  https://docs.github.com/en/pages/getting-started-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site):
  Pages source "GitHub Actions"; permissions `contents: read`, `pages: write`,
  `id-token: write`; `deploy-pages` runs only on pushes to the default branch.
  Latest releases (GitHub releases pages, 2026-10-18; years not shown on the
  page for the older ones):
  `actions/checkout` v7.0.1 (20 Jul), `actions/configure-pages` v6.0.0 (25
  Mar), `actions/upload-pages-artifact` v5.0.0 (10 Apr), `actions/deploy-pages`
  v5.0.1 (1 Sep), `actions/upload-artifact` v7.0.2 (7 Oct), `actions/cache`
  v6.1.0 (26 Jun), `cachix/install-nix-action` v31.11.2 (14 Oct),
  `cachix/cachix-action` v17 (Mar), `withastro/action` v6 (Astro's example,
  Node 24 default; skip it, Nix builds).
  Workflow: `install-nix-action`, `cachix-action` (pull from `linlog-prover`,
  push only own outputs), `nix build`, `upload-pages-artifact` with the output
  directory (`index.html` at top, a tar under 1 GB, no symlinks or hard links,
  so `cp -rL` out of the store first; dotfiles excluded by default,
  https://github.com/actions/upload-pages-artifact), then `deploy-pages` in the
  `github-pages` environment. `configure-pages` is optional with a fixed `site`.
  No `actions/cache` for `node_modules` is needed with Cachix.
- URLs (https://docs.github.com/en/pages/getting-started-with-github-pages/about-github-pages):
  a user or organization site comes from a repository named
  `<owner>.github.io` (lowercase), is served at `https://<owner>.github.io`, one
  per account; project sites stay at `https://<owner>.github.io/<repository>`.
  So `linlog-prover.github.io` serves the root (it returns 404 today) and the
  rustdoc stays at `/linlog`. Astro: `site: "https://linlog-prover.github.io"`,
  no `base` ("skip it if the repo is named `<username>.github.io`").
- Limits (https://docs.github.com/en/pages/getting-started-with-github-pages/github-pages-limits):
  1 GB site, 10-minute deployment timeout, 100 GB/month soft bandwidth, 429
  possible; the 10-builds-per-hour soft cap does not apply to custom workflows.
- `.nojekyll`: not needed; a custom workflow "can use a build process other
  than Jekyll" and Astro's `_astro/` is only a problem under Jekyll. Astro's
  guide never mentions it and the artifact step would drop a dotfile.
- `404.html` in the publishing source is served for unknown paths
  (https://docs.github.com/en/pages/getting-started-with-github-pages/creating-a-custom-404-page-for-your-github-pages-site);
  Astro builds `src/pages/404.astro` to it. Wanted for a friendly page; no SPA
  fallback is needed, the app has one URL and its state in the fragment.

## 6. Licence notices

- Vite 8 `build.license: boolean | { fileName?: string }`: `true` writes
  `.vite/license.md` listing the licences of all bundled dependencies; a
  `.json` file name gives raw metadata (https://vite.dev/config/build-options).
  The default is a dot directory, which the Pages artifact drops, so set
  `fileName: "third-party-licenses.md"` (in Astro under `vite.build.license`).
  Not verified: since which version, and whether it covers worker bundles and
  CSS; diff against `pnpm licenses list` once.
- Plugins (npm registry, 2026-10-18):
  `rollup-plugin-license` 3.7.1 (2026-04-09, MIT; peer rollup <= 4; README
  mentions neither Rolldown nor Vite; `thirdParty.allow` takes an SPDX
  expression and can fail the build);
  `rollup-license-plugin` 3.2.1 (2026-03-13; `createViteLicensePlugin()`;
  Vite 8 not verified);
  `vite-plugin-license` 0.0.2 (2025-11-08; peer `vite ^7.1.12`; avoid);
  `license-checker-rseidelsohn` 5.0.1 (2026-05-27, BSD-3-Clause; checks
  `node_modules`, not the bundle);
  `@rollup/plugin-license` does not exist (404).
  Recommendation: `build.license` for the served file and
  `license-checker-rseidelsohn --production --onlyAllow "<SPDX list>"` (or
  `rollup-plugin-license`'s `allow`) as a flake check against the same allow-list
  as `deny.toml`.
- **The Rust side is missing from every JS tool**: the wasm holds `linlog` and
  its crates, whose MIT/Apache notices must be served too. `cargo-about`
  (EmbarkStudios, crates.io 0.9.2, 2026-08-18; its Mozilla URL 404s) writes a
  notice from `Cargo.lock` with an accepted-licence list;
  `cargo-bundle-licenses` 4.2.0 (2025-10-21) is an alternative. Not verified:
  `cargo-about`'s target filtering for `wasm32-unknown-unknown`. Concatenate
  both outputs into the served file.
- **Euler Math**. `Euler-Math.otf` 0.75 (2026-02-18, packaged by Daniel Flipo,
  https://tug.ctan.org/fonts/euler-math/README.md) is "licensed under the SIL
  Open Font License, Version 1.1"; its other files are LPPL. I parsed the
  font's `name` table: ID 0 "Copyright (c) 1997, 2009 American Mathematical
  Society (http://www.ams.org). Copyright (c) 2009, 2021 Khaled Hosny
  (khaledhosny@eglug.org). Copyright (c) 2022-2026, Daniel Flipo
  (daniel.flipo@free.fr)", ID 13 "This Font Software is licensed under the SIL
  Open Font License, Version 1.1.", ID 14 `http://scripts.sil.org/OFL`, family
  "Euler Math". **No Reserved Font Name is declared**: the OFL defines an RFN
  as a name "specified after the copyright statement(s)" and none follows
  (https://openfontlicense.org/open-font-license-official-text/). The CTAN
  package carries no `OFL.txt`, and the RFN notice that turns up in a MathJax
  mirror belongs to another font (Neo Euler MathJax). Serving it under its own
  name is therefore fine. Ship `OFL.txt` (the 1.1 text, with the three copyright
  lines on top) beside the font file: condition 2 wants the notice and licence
  in each copy, which the name records and the file together satisfy.
- Subsetting and WOFF2 (https://openfontlicense.org/ofl-faq/): removing parts
  for a webfont counts as modification (2.6), which the OFL permits but "would
  not normally allow the use of RFNs"; a format change is normally
  modification unless only WOFF compression changes the data (2.2). With no
  RFN no rename is needed. Remaining duties: the subset stays under the OFL
  (condition 5), keeps the copyright and licence in its metadata (FAQ 2.4), no
  endorsement use of the authors' names (4), no sale by itself (1).
  `pyftsubset` keeps "only nameIDs between 0 and 6" by default
  (https://fonttools.readthedocs.io/en/latest/subset/index.html), which drops
  IDs 13 and 14, the licence records: pass `--name-IDs='*'` with
  `--flavor=woff2`. This is the foundry's reading of its licence, not legal
  advice.

## 7. Quality floors

WCAG 2.2 (https://www.w3.org/WAI/standards-guidelines/wcag/new-in-22/ and the
Understanding pages under https://www.w3.org/WAI/WCAG22/Understanding/):

| SC | Level | For a proof page |
|---|---|---|
| 2.1.1 Keyboard | A | every goal, formula, rule and the drawing's controls by keyboard |
| 2.4.7 Focus Visible | AA | visible focus on all controls and selectable formulas |
| 2.4.11 Focus Not Obscured (Min.) | AA | a focused item at least partly visible; sticky bars and phone sheets must not cover it |
| 2.4.13 Focus Appearance | **AAA** | 2 CSS px perimeter-sized, 3:1 change; a design target only |
| 2.5.7 Dragging Movements | AA | any drag (moving, resizing, panning) needs a single-pointer alternative without dragging; clicking a formula is one. Keyboard support alone does not satisfy it |
| 2.5.8 Target Size (Min.) | AA | 24x24 CSS px, or a 24 px circle clear of other targets; inline text targets exempt; pad small formula hit areas |
| 1.4.10 Reflow | AA | 320 CSS px wide without two-dimensional scrolling; "diagrams needed for understanding" may scroll in their own container, nothing else |
| 1.4.11 Non-text Contrast | AA | 3:1 for control edges, focus and state indicators, and graphic parts needed to understand (net links, selection); disabled exempt |
| 4.1.3 Status Messages | AA | verdicts, "rule applied", errors and search progress via `role="status"`/`role="alert"` or a live region, without moving focus |

(1.4.3 text contrast 4.5:1 and 1.4.4 resize text not re-read.)

- **Interactive SVG**: SVG-AAM (a draft that says it "contains outdated
  information", https://www.w3.org/TR/svg-aam-1.0/) leaves shapes out of the
  accessibility tree unless they have a `title`/`desc`, `aria-label`,
  `aria-labelledby`, a valid `tabindex` or a role; root is `graphics-document`,
  shapes `graphics-symbol`. The simple pattern for a drawing is `role="img"`
  with `<title>`, `<desc>` and `aria-labelledby`
  (https://tpgi.com/using-aria-enhance-svg-accessibility). But browsers
  treat every descendant of `role="img"` as presentational, so a button inside
  loses role and name
  (https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Roles/img_role).
  So: static exports get `role="img"` + title/desc; the interactive derivation
  is HTML (real `<button>`s, or `role="button"` + `tabindex="0"` + name +
  Enter/Space) with the SVG as the picture (`aria-hidden="true"` next to a
  text rendering for screen readers). `graphics-document` support is limited
  (search summary); test with a screen reader, not done here. Names: verb
  first, unique among siblings
  (https://www.w3.org/WAI/ARIA/apg/practices/names-and-descriptions/).
- Preferences: `prefers-reduced-motion` (`reduce`, widely available since
  Jan 2020), `prefers-contrast` (`more`, `less`, `custom`, since May 2022),
  `prefers-color-scheme` (not re-read), and `forced-colors: active` (use
  `currentColor` in SVG). MDN pages: .../CSS/@media/prefers-reduced-motion and
  .../prefers-contrast.
- Core Web Vitals at the 75th percentile, mobile and desktop separately:
  LCP <= 2.5 s, INP <= 200 ms (200-500 needs improvement, over 500 poor),
  CLS <= 0.1 (https://web.dev/articles/vitals , https://web.dev/articles/inp).
  INP covers click, tap and key interactions to the next paint; the worker keeps
  the main thread free, so the main-thread cost is handlers and inserting SVG:
  cap the size of an SVG inserted at once (`ViewOptions` already bounds it).
- Budget (https://web.dev/articles/your-first-performance-budget): Moto G4 on
  slow 3G, JS 100 KB, about 170 KB critical path, TTI 5 s; slow 4G, 200 KB JS,
  about 345 KB, TTI 3 s, compressed. The wasm (likely over 1 MB) exceeds that
  alone: show sequent form and text first, load the module from the worker with
  a status line and a progress bar from `Content-Length`. The critical path is
  then the Astro page, the Solid chunk and the font. Not verified: the article
  is old (2018 device); take it as a scale.
- Fonts (https://web.dev/articles/optimize-webfont-loading): woff2; preload the
  one font first paint needs (preload "offers no guarantees"); set
  `font-display`; subset; long cache lifetime (Pages sets 600 s, hashed names
  make that moot). For SVG text the font must be in the page's CSS
  (inline SVG); an SVG used as `<img>` cannot load external fonts (not
  verified here). Fallback glyphs have other widths: lay the SVG out from the
  library's measurements or use `font-display: block` for the short block
  period. Euler Math is 438 KB as shipped (full math table); the subset size
  is not measured.
- Privacy: Pages logs visitor IPs, "logged and stored for security purposes",
  signed in or not (the about-Pages page above). The general privacy statement
  does not mention Pages
  (https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement).
  With no analytics, no cookies and self-hosted fonts, the site adds nothing
  to that. Say so in one sentence in the footer: nothing typed leaves the
  browser; GitHub, which hosts the site, logs visitor IPs.

## 8. Precedents

- **Lean 4 web editor**: not one. The lean4web README says the Lean server runs
  on a web server inside Bubblewrap, not in the browser; only the old Lean 3
  editor (Emscripten) ran in the browser
  (https://github.com/leanprover-community/lean4web). This contradicts the
  premise in the brief.
- **Typst web app**: typst.ts compiles Typst in the browser and its README cites
  typst.app as an example (https://github.com/Myriad-Dreamin/typst.ts). Not
  verified: typst.app's own use of workers and offline; no official source.
- **egui/eframe demos**: Trunk, `data-wasm-opt="2"`, a CSS spinner "Loading…" in
  `index.html` removed by the Rust code, `<noscript>` text, a service worker
  registered on `load` (skipped for the hash `#dev`, a handy switch), no
  error handler for a failed load, no worker thread
  (https://github.com/emilk/eframe_template).
- **Pyodide**: a module worker, request-id protocol, interrupts only with
  `SharedArrayBuffer`; the nearest model for the protocol here.
- **Excalidraw**: `vite-plugin-pwa` (section 4), `assetsInlineLimit: 0` so
  fonts stay files, `manualChunks`.

## What could not be verified

Astro 7 with `@vite-pwa/astro`; `build.license` coverage of workers;
`worker.format` needs for wasm glue; Safari's `replaceState` limit; wasm memory
on current iOS; hello-world sizes of the Rust frameworks; typst.app internals;
`cargo-about` for wasm targets; `Uint8Array.toBase64` Baseline; whether GitHub
documents the shared origin of an organization's Pages sites. Search-tool
summaries stood in for primary pages on the WebKit bugs and browser URL limits.

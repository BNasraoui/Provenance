//! Vendored wiki stylesheet and theme switcher, ported from the design
//! mockup (`wiki-mockup-themes.html`). Everything is embedded in the
//! binary; generated pages make no external requests.
//!
//! The stylesheet is the mockup's `<style>` block verbatim — five theme
//! token blocks (statesman default, piano, latte, mocha, dracula) plus the
//! theme-agnostic grammar — followed by a small extensions section for the
//! page kinds and statuses the single mockup page did not need. All
//! extensions use the same `--pv-*` tokens, so every theme covers them.

/// The full wiki stylesheet, served as one vendored asset.
pub const WIKI_CSS: &str = r#"
    /* ============================================================
       THEME CONTRACT
       A theme is one block of custom properties. The default block
       is the Statesman Provenance atlas palette, verbatim from
       statesman-web/src/styles.css (--pv-*). Alternate themes remap
       the same token names; everything below the theme blocks is
       theme-agnostic grammar.
       ============================================================ */

    :root, :root[data-theme="statesman"] {
      color-scheme: light;
      --pv-canvas-bg: #faf8f5;
      --pv-canvas-dots: #e0dbd4;
      --pv-card-bg: #fffdf9;
      --pv-card-border: #e4ddd3;
      --pv-card-shadow: rgba(120, 100, 70, 0.08);
      --pv-ink: #2c2418;
      --pv-muted: #8a8278;

      --pv-source: #d4a574;
      --pv-source-bg: #fdf3e8;
      --pv-requirement: #6b9e7a;
      --pv-requirement-bg: #ecf5ef;
      --pv-resolution: #8b7bb5;
      --pv-resolution-bg: #f0ecf7;
      --pv-rule: #5a8f9e;
      --pv-rule-bg: #e8f2f5;
      --pv-thread: #8a8578;
      --pv-thread-bg: #f5f3ef;

      --pv-status-discovery: #b89e5a;
      --pv-status-refinement: #7c9eb2;
      --pv-status-resolved: #6b9e7a;
      --pv-approved: #5b8e6a;
      --pv-sev-high: #a85555;
      --pv-sev-medium: #a08040;

      --pv-wither-superseded: #d4a050;

      --pv-font-display: "Source Serif 4", "Source Serif Pro", Georgia, serif;
      --pv-font-body: "DM Sans", Cantarell, -apple-system, "Segoe UI", sans-serif;
      --pv-font-mono: "DM Mono", "JetBrains Mono", "Fira Code", ui-monospace, monospace;
    }

    :root[data-theme="piano"] {
      color-scheme: light;
      --pv-canvas-bg: #f7f4ec;
      --pv-canvas-dots: #e3ddcf;
      --pv-card-bg: #fffdf7;
      --pv-card-border: #ddd6c7;
      --pv-card-shadow: rgba(22, 19, 15, 0.07);
      --pv-ink: #16130f;
      --pv-muted: #8a8375;

      --pv-source: #6e6a61;
      --pv-source-bg: #f0eee8;
      --pv-requirement: #94651c;
      --pv-requirement-bg: #f6efe1;
      --pv-resolution: #2e2a24;
      --pv-resolution-bg: #edeae2;
      --pv-rule: #a3273b;
      --pv-rule-bg: #f8e9ec;
      --pv-thread: #8a8578;
      --pv-thread-bg: #f2efe7;

      --pv-status-discovery: #94651c;
      --pv-status-refinement: #6e6a61;
      --pv-status-resolved: #3d5c3a;
      --pv-approved: #3d5c3a;
      --pv-sev-high: #a3273b;
      --pv-sev-medium: #94651c;

      --pv-wither-superseded: #b08b3e;

      --pv-font-display: "Didot", "Bodoni MT", "Libre Bodoni", "Playfair Display", "Source Serif 4", Georgia, serif;
      --pv-font-body: "DM Sans", Cantarell, -apple-system, "Segoe UI", sans-serif;
      --pv-font-mono: "DM Mono", "JetBrains Mono", ui-monospace, monospace;
    }

    :root[data-theme="latte"] {
      color-scheme: light;
      --pv-canvas-bg: #eff1f5;
      --pv-canvas-dots: #dce0e8;
      --pv-card-bg: #ffffff;
      --pv-card-border: #ccd0da;
      --pv-card-shadow: rgba(76, 79, 105, 0.08);
      --pv-ink: #4c4f69;
      --pv-muted: #8c8fa1;

      --pv-source: #df8e1d;
      --pv-source-bg: #f9efdd;
      --pv-requirement: #40a02b;
      --pv-requirement-bg: #e8f3e4;
      --pv-resolution: #8839ef;
      --pv-resolution-bg: #f0e6fd;
      --pv-rule: #179299;
      --pv-rule-bg: #e0f1f2;
      --pv-thread: #7c7f93;
      --pv-thread-bg: #e6e9ef;

      --pv-status-discovery: #df8e1d;
      --pv-status-refinement: #209fb5;
      --pv-status-resolved: #40a02b;
      --pv-approved: #40a02b;
      --pv-sev-high: #d20f39;
      --pv-sev-medium: #df8e1d;

      --pv-wither-superseded: #df8e1d;

      --pv-font-display: "Source Serif 4", Georgia, serif;
      --pv-font-body: "DM Sans", Cantarell, -apple-system, "Segoe UI", sans-serif;
      --pv-font-mono: "DM Mono", "JetBrains Mono", ui-monospace, monospace;
    }

    :root[data-theme="mocha"] {
      color-scheme: dark;
      --pv-canvas-bg: #1e1e2e;
      --pv-canvas-dots: #313244;
      --pv-card-bg: #262637;
      --pv-card-border: #45475a;
      --pv-card-shadow: rgba(17, 17, 27, 0.6);
      --pv-ink: #cdd6f4;
      --pv-muted: #9399b2;

      --pv-source: #fab387;
      --pv-source-bg: #3a3243;
      --pv-requirement: #a6e3a1;
      --pv-requirement-bg: #2f3a44;
      --pv-resolution: #cba6f7;
      --pv-resolution-bg: #363150;
      --pv-rule: #94e2d5;
      --pv-rule-bg: #2c3a4a;
      --pv-thread: #9399b2;
      --pv-thread-bg: #313244;

      --pv-status-discovery: #f9e2af;
      --pv-status-refinement: #89b4fa;
      --pv-status-resolved: #a6e3a1;
      --pv-approved: #a6e3a1;
      --pv-sev-high: #f38ba8;
      --pv-sev-medium: #f9e2af;

      --pv-wither-superseded: #f9e2af;

      --pv-font-display: "Source Serif 4", Georgia, serif;
      --pv-font-body: "DM Sans", Cantarell, -apple-system, "Segoe UI", sans-serif;
      --pv-font-mono: "DM Mono", "JetBrains Mono", ui-monospace, monospace;
    }

    :root[data-theme="dracula"] {
      color-scheme: dark;
      --pv-canvas-bg: #282a36;
      --pv-canvas-dots: #3b3d4d;
      --pv-card-bg: #2e3040;
      --pv-card-border: #44475a;
      --pv-card-shadow: rgba(20, 21, 30, 0.6);
      --pv-ink: #f8f8f2;
      --pv-muted: #9aa3c7;

      --pv-source: #ffb86c;
      --pv-source-bg: #3d3a41;
      --pv-requirement: #50fa7b;
      --pv-requirement-bg: #2e3f3d;
      --pv-resolution: #bd93f9;
      --pv-resolution-bg: #383353;
      --pv-rule: #8be9fd;
      --pv-rule-bg: #2c3d4e;
      --pv-thread: #9aa3c7;
      --pv-thread-bg: #343747;

      --pv-status-discovery: #f1fa8c;
      --pv-status-refinement: #8be9fd;
      --pv-status-resolved: #50fa7b;
      --pv-approved: #50fa7b;
      --pv-sev-high: #ff5555;
      --pv-sev-medium: #f1fa8c;

      --pv-wither-superseded: #f1fa8c;

      --pv-font-display: "JetBrains Mono", "Cascadia Code", ui-monospace, monospace;
      --pv-font-body: "DM Sans", Cantarell, -apple-system, "Segoe UI", sans-serif;
      --pv-font-mono: "DM Mono", "JetBrains Mono", ui-monospace, monospace;
    }

    /* ============================================================
       GRAMMAR — mirrors the Statesman Provenance resolution detail
       page: type accent bar, 1040px container, icon + display title,
       780px body + 220px scholarly margin, tinted territory cards,
       full-width Field Notes band.
       ============================================================ */

    * { box-sizing: border-box; }

    body {
      margin: 0;
      background: var(--pv-canvas-bg);
      color: var(--pv-ink);
      font-family: var(--pv-font-body);
      font-size: 14px;
      line-height: 1.6;
    }

    a { color: inherit; }

    code {
      font-family: var(--pv-font-mono);
      font-size: 0.85em;
    }

    .icon { width: 1em; height: 1em; flex: none; stroke: currentColor; stroke-width: 2; stroke-linecap: round; stroke-linejoin: round; fill: none; }

    /* ---- top chrome ---- */

    .chrome {
      background: var(--pv-card-bg);
      border-bottom: 1px solid var(--pv-card-border);
      font-size: 13px;
    }

    .chrome-inner {
      max-width: 1040px;
      margin: 0 auto;
      padding: 0.55rem 1.5rem;
      display: flex;
      align-items: flex-start;
      gap: 1rem;
    }

    .wordmark {
      font-family: var(--pv-font-display);
      font-weight: 700;
      font-size: 15px;
      letter-spacing: -0.01em;
    }

    .wordmark .scope { font-family: var(--pv-font-mono); font-weight: 400; font-size: 11px; color: var(--pv-muted); margin-left: 0.5rem; }

    .chrome nav { display: flex; flex: 1 1 auto; min-width: 0; gap: 0.25rem; flex-wrap: wrap; align-items: center; color: var(--pv-muted); }
    .chrome nav a { text-decoration: none; padding: 0.15rem 0.4rem; border-radius: 6px; overflow-wrap: anywhere; }
    .chrome nav a:hover { background: var(--pv-thread-bg); color: var(--pv-ink); }
    .chrome nav .sep { opacity: 0.5; }

    .theme-select {
      margin-left: auto;
      display: flex;
      align-items: center;
      gap: 0.4rem;
      color: var(--pv-muted);
      font-size: 11px;
      text-transform: uppercase;
      letter-spacing: 0.08em;
    }

    .theme-select select {
      font: inherit;
      font-size: 12px;
      text-transform: none;
      letter-spacing: normal;
      color: var(--pv-ink);
      background: var(--pv-card-bg);
      border: 1px solid var(--pv-card-border);
      border-radius: 6px;
      padding: 0.2rem 0.4rem;
    }

    /* ---- accent bar (ATLAS-COLOR-SEMANTICS: page type) ---- */

    .accent-bar { height: 4px; background: var(--pv-requirement); }

    /* ---- page container ---- */

    .container { max-width: 1040px; margin: 0 auto; padding: 0 1.5rem; }

    .back-link {
      display: inline-flex;
      align-items: center;
      gap: 0.3rem;
      margin: 1.25rem 0 1rem;
      font-size: 13px;
      color: var(--pv-requirement);
      opacity: 0.7;
      text-decoration: none;
    }
    .back-link:hover { opacity: 1; }

    /* ---- title row ---- */

    .title-row { display: flex; align-items: flex-start; gap: 0.75rem; margin-bottom: 1.5rem; }
    .title-row > .icon { width: 24px; height: 24px; margin-top: 0.3rem; color: var(--pv-requirement); }

    h1 {
      font-family: var(--pv-font-display);
      font-size: 24px;
      font-weight: 700;
      letter-spacing: -0.02em;
      line-height: 1.3;
      margin: 0;
    }

    .badge-row { display: flex; align-items: center; gap: 0.5rem; margin-top: 0.6rem; flex-wrap: wrap; }

    .type-badge {
      display: inline-flex;
      align-items: center;
      gap: 0.35rem;
      padding: 0.25rem 0.5rem;
      font-size: 12px;
      font-weight: 500;
      border-radius: 6px;
      border: 1px solid var(--tb, var(--pv-thread));
      background: var(--tbg, var(--pv-thread-bg));
      color: var(--tb, var(--pv-thread));
    }
    .type-badge .icon { width: 14px; height: 14px; }
    .type-badge.requirement { --tb: var(--pv-requirement); --tbg: var(--pv-requirement-bg); }
    .type-badge.resolution  { --tb: var(--pv-resolution);  --tbg: var(--pv-resolution-bg); }
    .type-badge.rule        { --tb: var(--pv-rule);        --tbg: var(--pv-rule-bg); }
    .type-badge.source      { --tb: var(--pv-source);      --tbg: var(--pv-source-bg); }

    .status-badge {
      display: inline-flex;
      align-items: center;
      gap: 0.35rem;
      padding: 0.25rem 0.55rem;
      font-size: 12px;
      font-weight: 500;
      border-radius: 999px;
      color: var(--sc, var(--pv-muted));
      background: color-mix(in srgb, var(--sc, var(--pv-muted)) 15%, transparent);
      border: 1px solid color-mix(in srgb, var(--sc, var(--pv-muted)) 40%, transparent);
    }
    .status-badge .icon { width: 14px; height: 14px; }
    .status-badge.discovery { --sc: var(--pv-status-discovery); }
    .status-badge.approved  { --sc: var(--pv-approved); }

    .id-chip { font-family: var(--pv-font-mono); font-size: 11px; color: var(--pv-muted); }

    /* ---- two-column body: 780 + 220 ---- */

    .body-grid { display: flex; gap: 1.5rem; padding-bottom: 2rem; }
    .body-main { flex: 1; min-width: 0; max-width: 780px; }
    .body-margin { flex: none; width: 220px; }

    section { margin-bottom: 1.5rem; }

    .section-head {
      font-size: 12px;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.08em;
      margin: 0 0 0.5rem;
      color: var(--sh, var(--pv-muted));
      display: flex;
      align-items: center;
      gap: 0.4rem;
    }
    .section-head .icon { width: 14px; height: 14px; }
    .sh-requirement { --sh: var(--pv-requirement); }
    .sh-resolution  { --sh: var(--pv-resolution); }
    .sh-rule        { --sh: var(--pv-rule); }

    .prose { margin: 0; line-height: 1.65; }

    /* position — visually elevated, display serif */
    .position {
      margin: 0;
      background: var(--pv-resolution-bg);
      border-left: 4px solid var(--pv-resolution);
      border-radius: 8px;
      padding: 1rem;
      font-family: var(--pv-font-display);
      font-weight: 500;
      line-height: 1.6;
    }

    .decision-title {
      font-family: var(--pv-font-display);
      font-size: 16px;
      font-weight: 700;
      margin: 0 0 0.5rem;
    }
    .decision-title a { color: var(--pv-resolution); text-decoration: none; }
    .decision-title a:hover { text-decoration: underline; }

    /* territory cards */
    .territory { display: grid; grid-template-columns: 1fr; gap: 0.75rem; }

    .territory-card {
      border: 1px solid var(--tb, var(--pv-thread));
      background: var(--tbg, var(--pv-thread-bg));
      border-radius: 8px;
      padding: 0.75rem;
    }
    .territory-card.rule { --tb: var(--pv-rule); --tbg: var(--pv-rule-bg); }

    .territory-card .card-head {
      display: flex;
      align-items: center;
      gap: 0.35rem;
      font-size: 10px;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.08em;
      color: var(--tb);
      margin-bottom: 0.6rem;
    }
    .territory-card .card-head .icon { width: 14px; height: 14px; }

    .rule-list { list-style: none; margin: 0; padding: 0; }
    .rule-list li {
      display: grid;
      grid-template-columns: 7.5rem minmax(0, 1fr) auto;
      gap: 0.25rem 0.75rem;
      align-items: baseline;
      padding: 0.45rem 0;
      border-top: 1px solid color-mix(in srgb, var(--pv-rule) 18%, transparent);
      font-size: 12px;
    }
    .rule-list li:first-child { border-top: 0; padding-top: 0; }
    .rule-list .rcode { font-family: var(--pv-font-mono); font-weight: 500; color: var(--pv-rule); white-space: nowrap; }
    .rule-list .rname { font-weight: 600; }
    .rule-list .rstatement { grid-column: 2 / 4; color: var(--pv-muted); line-height: 1.5; }
    .rule-list .rmeta { display: flex; gap: 0.35rem; align-items: baseline; white-space: nowrap; }
    .rule-list .rref { grid-column: 2 / 4; font-family: var(--pv-font-mono); font-size: 10px; color: var(--pv-rule); opacity: 0.65; }

    .sev {
      font-size: 9px;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.06em;
      padding: 0.1rem 0.4rem;
      border-radius: 4px;
      color: var(--sv, var(--pv-muted));
      background: color-mix(in srgb, var(--sv, var(--pv-muted)) 12%, transparent);
    }
    .sev.high { --sv: var(--pv-sev-high); }
    .sev.medium { --sv: var(--pv-sev-medium); }
    .sev.modality { --sv: var(--pv-thread); }

    /* attribution */
    .attribution {
      border-top: 1px solid var(--pv-card-border);
      padding-top: 1rem;
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 0.5rem 1.5rem;
      font-size: 12px;
    }
    .attribution div { display: flex; align-items: center; gap: 0.4rem; }
    .attribution .icon { width: 12px; height: 12px; opacity: 0.4; }
    .attribution .k { color: var(--pv-muted); }
    .attribution .v { font-weight: 500; }

    /* ---- margin column: scholarly footnotes ---- */

    .margin-head {
      font-size: 10px;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.08em;
      color: var(--pv-resolution);
      margin: 0 0 0.75rem;
    }

    .citation { border-bottom: 1px solid var(--pv-card-border); padding-bottom: 0.75rem; margin-bottom: 0.75rem; }
    .citation .cite-head { display: flex; align-items: baseline; gap: 0.35rem; margin-bottom: 0.25rem; }
    .citation .cite-num { font-size: 10px; font-weight: 700; color: var(--pv-resolution); opacity: 0.6; font-variant-numeric: tabular-nums; }
    .citation .cite-type { font-size: 10px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.08em; color: var(--pv-resolution); opacity: 0.7; }
    .citation p { margin: 0; font-size: 12px; line-height: 1.55; }
    .citation .cite-ref { font-family: var(--pv-font-mono); font-size: 10px; color: var(--pv-resolution); opacity: 0.5; margin-top: 0.25rem; word-break: break-all; }

    .citation.gap {
      border: 1px dashed color-mix(in srgb, var(--pv-status-discovery) 55%, transparent);
      background: color-mix(in srgb, var(--pv-status-discovery) 8%, transparent);
      border-radius: 8px;
      padding: 0.6rem 0.7rem;
      color: color-mix(in srgb, var(--pv-status-discovery) 85%, var(--pv-ink));
    }
    .citation.gap p { font-size: 11px; }

    .classification { margin-top: 1.5rem; }
    .classification .row { display: flex; align-items: center; gap: 0.4rem; font-size: 11px; margin-bottom: 0.4rem; }
    .classification .icon { width: 12px; height: 12px; opacity: 0.4; }
    .classification .k { color: var(--pv-muted); }
    .classification .v { font-weight: 500; margin-left: auto; text-align: right; }
    .classification .v.mono { font-family: var(--pv-font-mono); font-size: 10px; }

    .lineage { margin-top: 1.5rem; }
    .lineage ol { list-style: none; margin: 0; padding: 0; }
    .lineage li { position: relative; padding: 0 0 0.6rem 0.9rem; font-size: 11px; line-height: 1.45; }
    .lineage li::before {
      content: "";
      position: absolute;
      left: 0; top: 0.35rem;
      width: 7px; height: 7px;
      border-radius: 50%;
      background: var(--pv-requirement);
      opacity: 0.55;
    }
    .lineage li:not(:last-child)::after {
      content: "";
      position: absolute;
      left: 3px; top: 0.85rem; bottom: -0.05rem;
      width: 1px;
      background: var(--pv-card-border);
    }
    .lineage li.current { font-weight: 600; }
    .lineage li.current::before { opacity: 1; }
    .lineage a { color: var(--pv-ink); opacity: 0.75; text-decoration: none; }
    .lineage a:hover { opacity: 1; text-decoration: underline; }

    /* ---- field notes band (full-width) ---- */

    .field-notes {
      border-top: 1px solid var(--pv-card-border);
      background: linear-gradient(180deg, color-mix(in srgb, var(--pv-thread-bg) 50%, transparent) 0%, color-mix(in srgb, var(--pv-canvas-bg) 20%, transparent) 100%);
    }

    .field-notes .container { padding-top: 1.25rem; padding-bottom: 1.5rem; }

    .fn-head { display: flex; align-items: center; gap: 0.5rem; margin-bottom: 1rem; }
    .fn-head .icon { width: 16px; height: 16px; color: var(--pv-thread); opacity: 0.5; }
    .fn-head h2 { font-family: var(--pv-font-display); font-size: 14px; font-weight: 600; margin: 0; }
    .fn-head .count { font-size: 11px; color: var(--pv-muted); }
    .fn-head .thread-id { font-family: var(--pv-font-mono); font-size: 10px; color: var(--pv-muted); opacity: 0.7; margin-left: auto; }

    .fn-list { max-width: 780px; }

    .field-note { display: flex; gap: 0.75rem; padding: 0.75rem 0; border-bottom: 1px solid var(--pv-card-border); }

    .role-badge {
      display: inline-flex;
      align-items: center;
      gap: 0.25rem;
      align-self: flex-start;
      margin-top: 0.15rem;
      padding: 0.15rem 0.4rem;
      font-size: 9px;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.08em;
      border-radius: 4px;
      background: var(--pv-rule-bg);
      color: var(--pv-rule);
      white-space: nowrap;
    }
    .role-badge .icon { width: 10px; height: 10px; }

    .field-note .fn-body { flex: 1; min-width: 0; }
    .field-note .fn-meta { display: flex; align-items: baseline; gap: 0.5rem; margin-bottom: 0.25rem; }
    .field-note .who { font-family: var(--pv-font-display); font-size: 11px; font-weight: 600; }
    .field-note time { font-size: 10px; color: var(--pv-muted); }
    .field-note .fn-content { margin: 0; white-space: pre-wrap; line-height: 1.55; font-size: 13px; }
    .field-note .fn-content .src { font-family: var(--pv-font-mono); font-size: 11px; color: var(--pv-thread); }

    .footer-note { color: var(--pv-muted); font-size: 11px; padding: 1rem 0 2rem; opacity: 0.8; }

    /* ---- responsive ---- */

    @media (max-width: 860px) {
      .body-grid { flex-direction: column; }
      .body-margin { width: auto; }
      .rule-list li { grid-template-columns: 1fr; }
      .rule-list .rstatement, .rule-list .rref { grid-column: 1; }
      .chrome nav { display: none; }
    }

    /* ============================================================
       RENDERER EXTENSIONS
       Additional page kinds, statuses, and list shapes the mockup
       page did not need, derived from the same theme tokens.
       ============================================================ */

    .accent-bar.scope-index { background: var(--pv-thread); }
    .accent-bar.resolution { background: var(--pv-resolution); }
    .accent-bar.rule { background: var(--pv-rule); }
    .accent-bar.source { background: var(--pv-source); }

    .back-link.scope-index, .title-row > .icon.scope-index { color: var(--pv-thread); }
    .back-link.resolution, .title-row > .icon.resolution { color: var(--pv-resolution); }
    .back-link.rule, .title-row > .icon.rule { color: var(--pv-rule); }
    .back-link.source, .title-row > .icon.source { color: var(--pv-source); }

    .status-badge.refinement { --sc: var(--pv-status-refinement); }
    .status-badge.active, .status-badge.resolved { --sc: var(--pv-status-resolved); }
    .status-badge.draft, .status-badge.review, .status-badge.proposed { --sc: var(--pv-status-discovery); }
    .status-badge.revised, .status-badge.superseded { --sc: var(--pv-wither-superseded); }
    .status-badge.rejected, .status-badge.abandoned { --sc: var(--pv-sev-high); }

    .sev.critical { --sv: var(--pv-sev-high); }
    .sev.low { --sv: var(--pv-thread); }

    .territory-card.requirement { --tb: var(--pv-requirement); --tbg: var(--pv-requirement-bg); }
    .territory-card.resolution { --tb: var(--pv-resolution); --tbg: var(--pv-resolution-bg); }
    .territory-card.source { --tb: var(--pv-source); --tbg: var(--pv-source-bg); }

    .sh-source { --sh: var(--pv-source); }
    .sh-thread { --sh: var(--pv-thread); }

    .link-list { list-style: none; margin: 0; padding: 0; }
    .link-list li { padding: 0.35rem 0; border-top: 1px solid color-mix(in srgb, var(--pv-card-border) 60%, transparent); font-size: 13px; }
    .link-list li:first-child { border-top: 0; padding-top: 0; }
    .link-list a { text-decoration: none; }
    .link-list a:hover { text-decoration: underline; }

    .index-list { list-style: none; margin: 0; padding: 0; }
    .index-list li { display: flex; align-items: baseline; gap: 0.6rem; flex-wrap: wrap; padding: 0.55rem 0; border-top: 1px solid var(--pv-card-border); }
    .index-list li:first-child { border-top: 0; padding-top: 0; }
    .index-list .entry-title { font-family: var(--pv-font-display); font-weight: 600; text-decoration: none; }
    .index-list .entry-title:hover { text-decoration: underline; }
    .index-list .entry-counts { font-family: var(--pv-font-mono); font-size: 10px; color: var(--pv-muted); margin-left: auto; white-space: nowrap; }

    .evidence-list { list-style: none; margin: 0; padding: 0; }
    .evidence-list li { font-family: var(--pv-font-mono); font-size: 12px; padding: 0.25rem 0; color: var(--pv-rule); }

    .orphan-card {
      border: 1px dashed color-mix(in srgb, var(--pv-status-discovery) 55%, transparent);
      background: color-mix(in srgb, var(--pv-status-discovery) 8%, transparent);
      border-radius: 8px;
      padding: 0.75rem;
    }
    .orphan-card h3 { font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.08em; margin: 0.6rem 0 0.25rem; color: color-mix(in srgb, var(--pv-status-discovery) 85%, var(--pv-ink)); }
    .orphan-card h3:first-child { margin-top: 0; }
"#;

/// The mockup's theme switcher, verbatim: applies the saved theme (or the
/// OS dark preference) and persists changes to `localStorage`.
pub const THEME_SCRIPT: &str = r#"
    (function () {
      var KEY = "provenance-wiki-theme";
      var root = document.documentElement;
      var select = document.getElementById("theme-select");

      function apply(theme) {
        root.setAttribute("data-theme", theme);
        select.value = theme;
      }

      var saved = null;
      try { saved = localStorage.getItem(KEY); } catch (e) {}
      if (saved) {
        apply(saved);
      } else if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
        apply("mocha");
      }

      select.addEventListener("change", function () {
        apply(select.value);
        try { localStorage.setItem(KEY, select.value); } catch (e) {}
      });
    })();
"#;

/// The mockup's inline icon sheet: one hidden `<svg>` of `<symbol>` defs
/// that pages reference with `<use href="#i-...">`.
pub const ICON_DEFS: &str = r#"<svg width="0" height="0" style="position:absolute" aria-hidden="true">
    <defs>
      <symbol id="i-git-branch" viewBox="0 0 24 24"><path d="M6 3v12"/><circle cx="18" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><path d="M18 9a9 9 0 0 1-9 9"/></symbol>
      <symbol id="i-scale" viewBox="0 0 24 24"><path d="m16 16 3-8 3 8c-.87.65-1.92 1-3 1s-2.13-.35-3-1Z"/><path d="m2 16 3-8 3 8c-.87.65-1.92 1-3 1s-2.13-.35-3-1Z"/><path d="M7 21h10"/><path d="M12 3v18"/><path d="M3 7h2c2 0 5-1 7-2 2 1 5 2 7 2h2"/></symbol>
      <symbol id="i-book-open" viewBox="0 0 24 24"><path d="M12 7v14"/><path d="M3 18a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h5a4 4 0 0 1 4 4 4 4 0 0 1 4-4h5a1 1 0 0 1 1 1v13a1 1 0 0 1-1 1h-6a3 3 0 0 0-3 3 3 3 0 0 0-3-3z"/></symbol>
      <symbol id="i-search" viewBox="0 0 24 24"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></symbol>
      <symbol id="i-check-circle" viewBox="0 0 24 24"><path d="M21.801 10A10 10 0 1 1 17 3.335"/><path d="m9 11 3 3L22 4"/></symbol>
      <symbol id="i-arrow-left" viewBox="0 0 24 24"><path d="m12 19-7-7 7-7"/><path d="M19 12H5"/></symbol>
      <symbol id="i-message-square" viewBox="0 0 24 24"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></symbol>
      <symbol id="i-user" viewBox="0 0 24 24"><path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></symbol>
      <symbol id="i-calendar" viewBox="0 0 24 24"><path d="M8 2v4"/><path d="M16 2v4"/><rect width="18" height="18" x="3" y="4" rx="2"/><path d="M3 10h18"/></symbol>
      <symbol id="i-shield" viewBox="0 0 24 24"><path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"/></symbol>
      <symbol id="i-gauge" viewBox="0 0 24 24"><path d="m12 14 4-4"/><path d="M3.34 19a10 10 0 1 1 17.32 0"/></symbol>
      <symbol id="i-bot" viewBox="0 0 24 24"><path d="M12 8V4H8"/><rect width="16" height="12" x="4" y="8" rx="2"/><path d="M2 14h2"/><path d="M20 14h2"/><path d="M15 13v2"/><path d="M9 13v2"/></symbol>
    </defs>
  </svg>
"#;
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css_carries_all_five_theme_token_blocks() {
        for theme in ["statesman", "piano", "latte", "mocha", "dracula"] {
            assert!(
                WIKI_CSS.contains(&format!("[data-theme=\"{theme}\"]")),
                "missing theme block: {theme}"
            );
        }
    }

    #[test]
    fn css_defines_the_atlas_type_tokens() {
        for token in [
            "--pv-source:",
            "--pv-requirement:",
            "--pv-resolution:",
            "--pv-rule:",
            "--pv-thread:",
        ] {
            assert!(WIKI_CSS.contains(token), "missing token: {token}");
        }
    }

    #[test]
    fn css_keeps_the_mockup_grammar() {
        for selector in [
            ".accent-bar",
            ".chrome-inner",
            ".body-grid",
            ".body-margin",
            ".territory-card",
            ".citation.gap",
            ".field-notes",
            ".role-badge",
            ".lineage",
        ] {
            assert!(WIKI_CSS.contains(selector), "missing selector: {selector}");
        }
    }

    #[test]
    fn css_allows_deep_breadcrumbs_to_wrap_in_chrome() {
        assert!(
            WIKI_CSS.contains(
                ".chrome-inner {\n      max-width: 1040px;\n      margin: 0 auto;\n      padding: 0.55rem 1.5rem;\n      display: flex;\n      align-items: flex-start;\n      gap: 1rem;\n    }"
            ),
            "chrome header should align cleanly when breadcrumbs wrap"
        );
        assert!(
            WIKI_CSS.contains(".chrome nav { display: flex; flex: 1 1 auto; min-width: 0;"),
            "breadcrumb nav should be shrink-safe inside the chrome flex row"
        );
        assert!(
            WIKI_CSS.contains("overflow-wrap: anywhere;"),
            "breadcrumb links should not force horizontal overflow"
        );
    }

    #[test]
    fn css_makes_no_external_requests() {
        assert!(!WIKI_CSS.contains("http://"));
        assert!(!WIKI_CSS.contains("https://"));
        assert!(!WIKI_CSS.contains("@import"));
        assert!(!WIKI_CSS.contains("url("));
    }

    #[test]
    fn theme_script_persists_the_choice_to_local_storage() {
        assert!(THEME_SCRIPT.contains("provenance-wiki-theme"));
        assert!(THEME_SCRIPT.contains("localStorage"));
        assert!(THEME_SCRIPT.contains("data-theme"));
    }

    #[test]
    fn icon_defs_cover_the_symbols_the_renderer_uses() {
        for symbol in [
            "i-git-branch",
            "i-scale",
            "i-book-open",
            "i-search",
            "i-check-circle",
            "i-arrow-left",
            "i-message-square",
            "i-user",
            "i-calendar",
            "i-shield",
            "i-gauge",
            "i-bot",
        ] {
            assert!(
                ICON_DEFS.contains(&format!("id=\"{symbol}\"")),
                "missing icon: {symbol}"
            );
        }
    }
}

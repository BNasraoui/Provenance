pub(super) const CSS_RESOLUTION_PAGE: &str = r#"    /* ============================================================
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
    .lineage a { color: var(--pv-requirement); opacity: 0.85; text-decoration: none; }
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

"#;

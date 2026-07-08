pub(super) const CSS_RENDERER_EXTENSIONS: &str = r"    /* ============================================================
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

    section:has(> .sh-requirement) { --pv-link-color: var(--pv-requirement); }
    section:has(> .sh-resolution) { --pv-link-color: var(--pv-resolution); }
    section:has(> .sh-rule) { --pv-link-color: var(--pv-rule); }
    section:has(> .sh-source) { --pv-link-color: var(--pv-source); }

    .link-list { list-style: none; margin: 0; padding: 0; }
    .link-list li { padding: 0.35rem 0; border-top: 1px solid color-mix(in srgb, var(--pv-card-border) 60%, transparent); font-size: 13px; }
    .link-list li:first-child { border-top: 0; padding-top: 0; }
    .link-list a { color: var(--pv-link-color, var(--pv-resolution)); text-decoration: none; }
    .link-list a:hover, .link-list a:focus-visible { text-decoration: underline; }

    .index-list { list-style: none; margin: 0; padding: 0; }
    .index-list li { display: flex; align-items: baseline; gap: 0.6rem; flex-wrap: wrap; padding: 0.55rem 0; border-top: 1px solid var(--pv-card-border); }
    .index-list li:first-child { border-top: 0; padding-top: 0; }
    .index-list .entry-title { color: var(--pv-requirement); font-family: var(--pv-font-display); font-weight: 600; text-decoration: none; }
    .index-list .entry-title:hover, .index-list .entry-title:focus-visible { text-decoration: underline; }
    .index-list .entry-counts { font-family: var(--pv-font-mono); font-size: 10px; color: var(--pv-muted); margin-left: auto; white-space: nowrap; }

    .evidence-list { list-style: none; margin: 0; padding: 0; }
    .evidence-list li { font-family: var(--pv-font-mono); font-size: 12px; padding: 0.25rem 0; color: var(--pv-rule); }

    .orphan-card {
      --pv-link-color: color-mix(in srgb, var(--pv-status-discovery) 85%, var(--pv-ink));
      border: 1px dashed color-mix(in srgb, var(--pv-status-discovery) 55%, transparent);
      background: color-mix(in srgb, var(--pv-status-discovery) 8%, transparent);
      border-radius: 8px;
      padding: 0.75rem;
    }
    .orphan-card h3 { font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.08em; margin: 0.6rem 0 0.25rem; color: color-mix(in srgb, var(--pv-status-discovery) 85%, var(--pv-ink)); }
    .orphan-card h3:first-child { margin-top: 0; }
";

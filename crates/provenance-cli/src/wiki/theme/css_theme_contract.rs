pub(super) const CSS_THEME_CONTRACT: &str = r#"
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

      --pv-source: #9c6730;
      --pv-source-bg: #fdf3e8;
      --pv-requirement: #517c5e;
      --pv-requirement-bg: #ecf5ef;
      --pv-resolution: #7a68aa;
      --pv-resolution-bg: #f0ecf7;
      --pv-rule: #4c7986;
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

      --pv-source: #996214;
      --pv-source-bg: #f9efdd;
      --pv-requirement: #327d22;
      --pv-requirement-bg: #e8f3e4;
      --pv-resolution: #8839ef;
      --pv-resolution-bg: #f0e6fd;
      --pv-rule: #137a80;
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

"#;

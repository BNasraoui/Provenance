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

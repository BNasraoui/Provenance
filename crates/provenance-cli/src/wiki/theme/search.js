(function () {
  var input = document.getElementById("wiki-search");
  var results = document.getElementById("search-results");
  var summary = document.getElementById("search-summary");
  if (!input || !results || !summary) return;
  var items = Array.prototype.slice.call(results.querySelectorAll("[data-search-entry]"));

  function update() {
    var query = input.value.trim().toLocaleLowerCase();
    var terms = query.split(/\s+/).filter(Boolean);
    var matches = [];
    items.forEach(function (item, position) {
      var title = (item.getAttribute("data-search-title") || "").toLocaleLowerCase();
      var statement = (item.getAttribute("data-search-statement") || "").toLocaleLowerCase();
      var matched = terms.length > 0 && terms.every(function (term) {
        return title.indexOf(term) !== -1 || statement.indexOf(term) !== -1;
      });
      item.hidden = !matched;
      if (matched) {
        var score = terms.reduce(function (total, term) {
          return total + (title.indexOf(term) !== -1 ? 2 : 1);
        }, title === query ? 10 : 0);
        matches.push({ item: item, score: score, position: position });
      }
    });
    matches.sort(function (left, right) {
      return right.score - left.score || left.position - right.position;
    });
    matches.forEach(function (match) { results.appendChild(match.item); });
    summary.textContent = terms.length === 0
      ? "Type one or more words to search titles and statements."
      : matches.length + (matches.length === 1 ? " result" : " results");
    try {
      var url = new URL(window.location.href);
      if (query) url.searchParams.set("q", input.value.trim());
      else url.searchParams.delete("q");
      history.replaceState(null, "", url);
    } catch (error) {}
  }

  try { input.value = new URL(window.location.href).searchParams.get("q") || ""; }
  catch (error) {}
  input.addEventListener("input", update);
  update();
})();

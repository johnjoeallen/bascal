// Light/dark theme toggle. Defaults to following the OS/browser
// preference (handled purely by CSS media query) until the user clicks
// the toggle, at which point the explicit choice is remembered and wins
// over system preference on every page, via a `data-theme` attribute
// set on <html>. The early inline script in <head> applies any stored
// choice before first paint, avoiding a flash of the wrong theme; this
// file just wires up the button and keeps its icon in sync.
(function () {
  "use strict";

  var STORAGE_KEY = "bascal-theme";

  function storedTheme() {
    try {
      return localStorage.getItem(STORAGE_KEY);
    } catch (e) {
      return null;
    }
  }

  function systemPrefersDark() {
    return (
      window.matchMedia &&
      window.matchMedia("(prefers-color-scheme: dark)").matches
    );
  }

  function effectiveTheme() {
    return storedTheme() || (systemPrefersDark() ? "dark" : "light");
  }

  function setTheme(value) {
    document.documentElement.setAttribute("data-theme", value);
    try {
      localStorage.setItem(STORAGE_KEY, value);
    } catch (e) {}
  }

  function syncButton(btn) {
    var eff = effectiveTheme();
    btn.textContent = eff === "dark" ? "☀️" : "\u{1F319}"; // sun / crescent moon
    btn.setAttribute(
      "aria-label",
      eff === "dark" ? "Switch to light theme" : "Switch to dark theme"
    );
  }

  function run() {
    var btn = document.getElementById("theme-toggle");
    if (!btn) return;
    syncButton(btn);
    btn.addEventListener("click", function () {
      setTheme(effectiveTheme() === "dark" ? "light" : "dark");
      syncButton(btn);
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", run);
  } else {
    run();
  }
})();

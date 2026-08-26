// Turn the compact Markdown status markers into coloured, accessible buttons.
document.addEventListener("DOMContentLoaded", () => {
  document.querySelectorAll("table").forEach((table) => {
    table.querySelectorAll("tbody td").forEach((cell) => {
      const value = cell.textContent.trim().toUpperCase();
      const kind = value === "PASS" ? "pass" : value === "FAIL" ? "fail" : value === "N/A" ? "na" : null;
      if (!kind) return;
      cell.innerHTML = `<span class="conformance-status ${kind}" role="status">${value}</span>`;
    });
  });
});

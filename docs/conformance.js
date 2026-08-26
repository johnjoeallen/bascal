// Turn every compact Markdown status marker into a coloured, accessible button.
document.addEventListener("DOMContentLoaded", () => {
  document.querySelectorAll("table").forEach((table) => {
    table.querySelectorAll("tbody td").forEach((cell) => {
      const value = cell.textContent.trim().toUpperCase();
      // PASS and known expected outcomes are good (green).  FAIL is an
      // observed unexpected result (red); an absent/unclassified result is
      // unknown (grey).  Keep the label itself so the distinction remains
      // visible without colour perception.
      const expected = new Set([
        "PASS", "DEFERRED", "UNSUPPORTED", "WILL NOT IMPLEMENT",
        "NOT APPLICABLE", "N/A"
      ]);
      const kind = value === "FAIL" ? "fail"
        : value === "DEFERRED" ? "deferred"
        : value === "NOT APPLICABLE" || value === "N/A" ? "not-applicable"
        : value === "UNSUPPORTED" || value === "WILL NOT IMPLEMENT" ? "unsupported"
        : expected.has(value) ? "pass"
        : value === "UNKNOWN" ? "unknown"
        : null;
      if (!kind) return;
      cell.innerHTML = `<span class="conformance-status ${kind}" role="status" aria-label="${value}">${value}</span>`;
    });
  });
});

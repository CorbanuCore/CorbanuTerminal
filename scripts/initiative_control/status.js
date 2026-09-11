"use strict";
let generation = document.body.dataset.generation || null;
const freshness = document.getElementById("freshness");
async function updateHealth() {
  try {
    const response = await fetch("health.json", {cache: "no-store"});
    if (!response.ok) throw new Error("unavailable");
    const health = await response.json();
    if (!health.ok) {
      freshness.className = "freshness failed";
      freshness.textContent = "Publisher failed · last successful snapshot retained · manager attention required";
      return;
    }
    const publishedGeneration = health.generation || health.published_at;
    if (generation && generation !== publishedGeneration) { location.reload(); return; }
    generation = publishedGeneration;
    const age = Math.max(0, Math.floor((Date.now() - Date.parse(document.body.dataset.collected)) / 60000));
    const publishAge = Math.max(0, Math.floor((Date.now() - Date.parse(health.published_at)) / 60000));
    const stale = !Number.isFinite(age) || age > 45 || publishAge > 45;
    freshness.className = "freshness" + (stale ? " stale" : "");
    freshness.textContent = `${stale ? "STALE · " : ""}Source collected ${age} min ago · published ${publishAge} min ago · refresh every 30 min${health.warning_count ? " · data warnings present" : ""}`;
    for (const item of document.querySelectorAll("time[data-seen]")) {
      const minutes = Math.floor((Date.now() - Date.parse(item.dataset.seen)) / 60000);
      item.textContent = `${item.dataset.seen} (${minutes} min ago${minutes > 45 ? "; stale" : ""})`;
    }
  } catch (_) {
    freshness.className = "freshness failed";
    freshness.textContent = "Connection unavailable · displayed snapshot may be stale · reopen the SSH connection using the runbook";
  }
}
updateHealth();
setInterval(updateHealth, 30000);

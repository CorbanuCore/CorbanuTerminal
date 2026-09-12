"use strict";
let generation = document.body.dataset.generation || null;
const freshness = document.getElementById("freshness");
function updateDecisionAge() {
  const section = document.getElementById("decisions");
  if (!section || !section.dataset.assessedAt) return; // Unknown never becomes empty.
  const oldest = document.getElementById("oldest-open-decision-age");
  if (oldest) {
    const elapsed = Date.now() - Date.parse(oldest.dataset.raisedAt);
    oldest.textContent = Number.isFinite(elapsed) && elapsed >= 0
      ? `Oldest raised ${Math.floor(elapsed / 60000)} minutes ago.`
      : "Oldest raised age unknown.";
  }
  const expired = stamp => Date.now() - Date.parse(stamp) > Number(section.dataset.freshSeconds) * 1000;
  const stale = expired(section.dataset.assessedAt);
  for (const p of section.querySelectorAll("p")) {
    // Only generated freshness text changes; user context and link labels stay intact.
    if (stale && p.parentElement === section && p.textContent.startsWith("Source ")) {
      p.textContent = p.textContent.replace("Fresh manager assessment.", "Stale: current decisions unknown; showing last-known records.");
    }
    if (stale && p.parentElement === section && p.textContent.startsWith("Open decisions:")) {
      p.firstChild.textContent = p.firstChild.textContent
        .replace("Open decisions:", "Last-known open:")
        .replace(" No open decisions found in this fresh assessment.", "");
    }
    const context = !p.firstElementChild && p.textContent.match(/^Revision \d+: \w+; raised [0-9TZ:-]+; context updated ([0-9TZ:-]+)/);
    if (context && (stale || expired(context[1]))) {
      p.textContent = p.textContent
        .replace("Context within freshness window.", "Stale context: stopped/continuing work and other details are last-known.");
    }
    const evidence = p.firstElementChild?.tagName === "A" && p.lastChild.textContent.match(/; assessed ([0-9TZ:-]+)$/);
    if (evidence && (stale || expired(evidence[1]))) {
      p.lastChild.textContent = p.lastChild.textContent
        .replace("Evidence within freshness window", "Stale evidence");
    }
  }
}
function revealDecisionLink() {
  const target = document.getElementById(location.hash.slice(1));
  if (!target || !target.closest("#decisions")) return;
  for (let item = target; item; item = item.parentElement) {
    if (item.tagName === "DETAILS") item.open = true;
  }
}
window.addEventListener("hashchange", revealDecisionLink);
revealDecisionLink();
function updateActivityAge() {
  for (const item of document.querySelectorAll("[data-activity-seen]")) {
    if (Date.now() - Date.parse(item.dataset.activitySeen) > 45 * 60000) {
      item.className = "badge stale";
      item.textContent = "Stale report — last: " + item.dataset.activityLabel;
    }
  }
}
async function updateHealth() {
  updateActivityAge();
  updateDecisionAge(); // Independent of publisher success or connection failure.
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
    const stale = !Number.isFinite(age) || age > 20 || publishAge > 45;
    freshness.className = "freshness" + (stale ? " stale" : "");
    freshness.textContent = `${stale ? "STALE · " : ""}Source collected ${age} min ago · published ${publishAge} min ago · manager sync every 10 min${health.warning_count ? " · data warnings present" : ""}`;
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

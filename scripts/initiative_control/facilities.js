"use strict";

(function () {
  const root = document.querySelector("[data-facilities-root]");
  if (!root) return;

  const endpoint = root.dataset.controlEndpoint;
  const statusLabels = {
    running: "Running",
    stopped: "Stopped",
    degraded: "Degraded",
    initializing: "Initializing",
    unreachable: "Unreachable",
    unknown: "Unknown",
  };
  let message = root.querySelector("[data-facility-message]");
  let actionPending = false;
  if (!message) {
    message = document.createElement("p");
    message.className = "facility-message muted";
    message.dataset.facilityMessage = "";
    message.setAttribute("aria-live", "polite");
    root.prepend(message);
  }
  const refreshMessage = document.createElement("p");
  refreshMessage.className = "facility-message muted";
  refreshMessage.dataset.facilityRefresh = "";
  root.append(refreshMessage);

  function containers(id) {
    return Array.from(document.querySelectorAll("[data-facility-card], [data-facility-row]"))
      .filter((item) => item.dataset.facilityId === id);
  }

  function update(record) {
    const label = statusLabels[record.status] || "Unknown";
    containers(record.id).forEach((container) => {
      container.querySelectorAll("[data-facility-status]").forEach((status) => {
        status.textContent = label;
        status.className = "facility-status status-" + (statusLabels[record.status] ? record.status : "unknown");
      });
      const detail = container.querySelector("[data-facility-detail]");
      if (detail && record.detail) detail.textContent = record.detail;
      container.querySelectorAll("[data-facility-action]").forEach((button) => {
        const action = button.dataset.facilityAction;
        button.disabled = actionPending || !(record.actions && record.actions[action]);
      });
    });
  }

  function setUnavailable(detail) {
    document.querySelectorAll("[data-facility-card], [data-facility-row]").forEach((container) => {
      const status = container.querySelector("[data-facility-status]");
      if (status) {
        status.textContent = "Unavailable";
        status.className = "facility-status status-unreachable";
      }
      const description = container.querySelector("[data-facility-detail]");
      if (description) description.textContent = detail;
      container.querySelectorAll("[data-facility-action]").forEach((button) => {
        button.disabled = true;
      });
    });
  }

  async function refresh() {
    try {
      const response = await fetch(endpoint + "/status", { cache: "no-store", credentials: "omit" });
      const payload = await response.json();
      if (!response.ok || !payload.ok || !payload.facilities) throw new Error(payload.error || "Status request failed.");
      Object.values(payload.facilities).forEach(update);
      refreshMessage.textContent = "Live status checked " + new Date().toLocaleTimeString() + ".";
    } catch (error) {
      const detail = "Local control bridge unavailable; reopen the Corbanu Control shortcut.";
      setUnavailable(detail);
      refreshMessage.textContent = detail;
    }
  }

  async function perform(button) {
    if (actionPending) return;
    const action = button.dataset.facilityAction;
    const id = button.dataset.facilityId;
    if (!window.confirm(action === "stop" ? "Stop this facility service?" : "Start this facility service?")) return;
    actionPending = true;
    document.querySelectorAll("[data-facility-action]").forEach((item) => { item.disabled = true; });
    message.textContent = action === "stop" ? "Stopping service…" : "Starting service…";
    try {
      const response = await fetch(endpoint + "/action", {
        method: "POST",
        cache: "no-store",
        credentials: "omit",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id: id, action: action }),
      });
      const payload = await response.json();
      if (!response.ok || !payload.ok) throw new Error(payload.error || payload.message || "Service action failed.");
      if (payload.facility) update(payload.facility);
      message.textContent = payload.message || "Service action completed.";
    } catch (error) {
      message.textContent = error.message || "Service action failed.";
    } finally {
      actionPending = false;
      await refresh();
    }
  }

  document.querySelectorAll("[data-facility-action]").forEach((button) => {
    button.addEventListener("click", () => perform(button));
    button.disabled = true;
  });
  refresh();
  window.setInterval(refresh, 15000);
}());

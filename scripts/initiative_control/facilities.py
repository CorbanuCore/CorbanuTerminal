"""Private facilities index and the service metadata used by its local bridge."""
from html import escape
from urllib.parse import urlsplit


# Link-only integration: listing this UI does not add bridge action authority.
MUSIC_STUDIO_URL = "http://100.99.88.49:7864/"


FACILITIES = (
    {
        "id": "comfyui",
        "name": "ComfyUI",
        "machine": "RTX PRO 6000",
        "purpose": "Image and workflow UI",
        "url": "http://100.99.88.49:8188/",
        "upstream": "https://github.com/comfyanonymous/ComfyUI",
        "host": "100.99.88.49",
        "service_units": ("comfyui-ui.service",),
    },
    {
        "id": "yue2",
        "name": "YuE2 (YuE)",
        "machine": "RTX PRO 6000",
        "purpose": "Lyrics-to-song generation",
        "url": "http://100.99.88.49:7861/",
        "upstream": "https://github.com/multimodal-art-projection/YuE",
        "host": "100.99.88.49",
        "service_units": ("yue2-ui.service",),
    },
    {
        "id": "ace-step-rtx",
        "name": "ACE-Step",
        "machine": "RTX PRO 6000",
        "purpose": "Music generation",
        "url": "http://100.99.88.49:7862/",
        "upstream": "https://github.com/ace-step/ACE-Step-1.5",
        "host": "100.99.88.49",
        "service_units": ("ace-step-ui.service",),
    },
    {
        "id": "minimax-music3",
        "name": "MiniMax Music 3",
        "machine": "RTX PRO 6000",
        "purpose": "Lyrics-to-song generation",
        "url": "http://100.99.88.49:7863/",
        "upstream": "https://huggingface.co/MiniMaxAI/MiniMax-Music3",
        "host": "100.99.88.49",
        "service_units": ("minimax-music3-api.service", "minimax-music3-ui.service"),
        "initializing_units": ("minimax-music3-download.service", "minimax-music3-activate.service"),
        "detail": "SGLang-Omni API plus browser UI; the checkpoint downloader is shown as initializing.",
    },
    {
        "id": "rvc",
        "name": "RVC",
        "machine": "Drone / RTX 4090",
        "purpose": "Voice conversion",
        "url": "http://100.81.145.102:7865/",
        "upstream": "https://github.com/RVC-Project/Retrieval-based-Voice-Conversion-WebUI",
        "host": "100.81.145.102",
        "service_units": ("rvc-ui.service",),
    },
    {
        "id": "ace-step-drone",
        "name": "ACE-Step fallback",
        "machine": "Drone / RTX 4090",
        "purpose": "Music generation fallback",
        "url": "http://100.81.145.102:7866/",
        "upstream": "https://github.com/ace-step/ACE-Step-1.5",
        "host": "100.81.145.102",
        "service_units": ("ace-step-ui-drone.service",),
    },
)


def _button(facility_id, action):
    label = action.capitalize()
    return (f'<button type="button" class="facility-action" '
            f'data-facility-action="{escape(action)}" data-facility-id="{escape(facility_id)}" '
            f'aria-label="{label} {escape(facility_id)}">{label}</button>')


def facilities():
    body = ('<section class="heading"><div><p class="eyebrow">SYSTEMS / MEDIA</p><h1>Facilities</h1>'
            '<p class="muted">Media-generation interfaces and service controls, grouped by machine.</p></div></section>')
    body += ('<aside class="notice"><strong>Private operator controls</strong> Status is read from the machines through a '
             'loopback-only control bridge started with the dashboard. Start and stop actions are explicit POST operations; '
             'the dashboard host never stores machine passwords. An unavailable machine remains unavailable rather than being '
             'treated as stopped.</aside>')
    body += ('<section data-facilities-root data-control-endpoint="http://127.0.0.1:8770">'
             '<div class="section-title"><h2>Machine interfaces</h2><span>7 registered interfaces · Live service status for 6 · refreshes every 15 seconds</span></div>'
             '<div class="test-grid">')
    for item in FACILITIES:
        detail = item.get("detail", "Service status and controls are provided by the local operator bridge.")
        body += (f'<article class="test facility-card" data-facility-card data-facility-id="{escape(item["id"])}">'
                 f'<div class="lane-top"><span class="facility-status status-checking" data-facility-status>Checking…</span>'
                 f'<span class="muted">{escape(item["machine"])}</span></div>'
                 f'<h3><a href="{escape(item["url"])}" rel="noreferrer noopener">{escape(item["name"])}</a></h3>'
                 f'<p>{escape(item["purpose"])}</p><p class="muted" data-facility-detail>{escape(detail)}</p>'
                 f'<div class="facility-controls" role="group" aria-label="{escape(item["name"])} service controls">'
                 f'{_button(item["id"], "start")}{_button(item["id"], "stop")}'
                 f'<a class="facility-open" href="{escape(item["url"])}" rel="noreferrer noopener">Open interface ↗</a>'
                 '</div>'
                 f'<small><a href="{escape(item["upstream"])}" rel="noreferrer noopener">Upstream repository</a></small></article>')
    body += ('<article class="test facility-card" id="music-studio">'
             '<div class="lane-top"><span class="facility-status">Link only</span>'
             '<span class="muted">RTX PRO 6000</span></div>'
             f'<h3><a href="{MUSIC_STUDIO_URL}" rel="noreferrer noopener">Music Studio</a></h3>'
             '<p>Shared music workspace for Yue2, MiniMax Music 3 and ACE-Step.</p>'
             '<p class="muted">Private Tailscale access required. Live status and start/stop controls '
             'are not connected to this dashboard.</p>'
             f'<div class="facility-controls"><a class="facility-open" href="{MUSIC_STUDIO_URL}" '
             'rel="noreferrer noopener">Open Music Studio ↗</a></div>'
             '<small>Private wrapper; no upstream repository.</small></article>')
    body += '</div></section><section><h2>At a glance</h2><div class="table-wrap"><table><thead><tr><th>Facility</th><th>Machine</th><th>Address / port</th><th>Primary use</th><th>Status</th></tr></thead><tbody>'
    for item in FACILITIES:
        body += (f'<tr data-facility-row data-facility-id="{escape(item["id"])}">'
                 f'<td><a href="{escape(item["url"])}" rel="noreferrer noopener">{escape(item["name"])}</a></td>'
                 f'<td>{escape(item["machine"])}</td><td>{escape(urlsplit(item["url"]).netloc)}</td>'
                 f'<td>{escape(item["purpose"])}</td><td><span class="facility-status status-checking" data-facility-status>Checking…</span></td></tr>')
    body += (f'<tr><td><a href="{MUSIC_STUDIO_URL}" rel="noreferrer noopener">Music Studio</a></td>'
             '<td>RTX PRO 6000</td><td>100.99.88.49:7864</td><td>Shared music workspace</td>'
             '<td>Link only · not monitored</td></tr>')
    return body + '</tbody></table></div></section>'

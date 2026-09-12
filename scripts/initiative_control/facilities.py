"""Static private operator index; importing or rendering never probes services."""
from html import escape
from urllib.parse import urlsplit

ENTRIES = (
    ("ComfyUI", "RTX PRO 6000", "Image and workflow UI", "http://100.99.88.49:8188/", "https://github.com/comfyanonymous/ComfyUI"),
    ("YuE2 (YuE)", "RTX PRO 6000", "Lyrics-to-song generation", "http://100.99.88.49:7861/", "https://github.com/multimodal-art-projection/YuE"),
    ("ACE-Step", "RTX PRO 6000", "Music generation", "http://100.99.88.49:7862/", "https://github.com/ace-step/ACE-Step-1.5"),
    ("MiniMax Music 3", "RTX PRO 6000", "Lyrics-to-song generation", "http://100.99.88.49:7863/", "https://huggingface.co/MiniMaxAI/MiniMax-Music3"),
    ("RVC", "Drone / RTX 4090", "Voice conversion", "http://100.81.145.102:7865/", "https://github.com/RVC-Project/Retrieval-based-Voice-Conversion-WebUI"),
    ("ACE-Step fallback", "Drone / RTX 4090", "Music generation fallback", "http://100.81.145.102:7866/", "https://github.com/ace-step/ACE-Step-1.5"),
)


def facilities():
    body = '<section class="heading"><div><p class="eyebrow">SYSTEMS / MEDIA</p><h1>Facilities</h1><p class="muted">Media-generation interfaces and upstream projects, grouped by machine.</p></div></section>'
    body += '<aside class="notice"><strong>Private operator index</strong> These links require access to the machines\' private network. Service availability is not checked by this dashboard. If a link is unavailable, check your network access and the machine or service; this page does not start or configure models.</aside>'
    body += '<section><div class="section-title"><h2>Machine interfaces</h2><span>6 registered interfaces · not a live health check</span></div><div class="test-grid">'
    for name, machine, purpose, url, upstream in ENTRIES:
        body += f'<article class="test"><small>{escape(machine)}</small><h3><a href="{escape(url)}" rel="noreferrer noopener">{escape(name)}</a></h3><p>{escape(purpose)}</p><p><a href="{escape(url)}" rel="noreferrer noopener">Open {escape(name)} interface ↗</a></p><small><a href="{escape(upstream)}" rel="noreferrer noopener">{escape(name)} upstream repository</a></small></article>'
    body += '</div></section><section><h2>At a glance</h2><div class="table-wrap"><table><thead><tr><th>Facility</th><th>Machine</th><th>Address / port</th><th>Primary use</th></tr></thead><tbody>'
    for name, machine, purpose, url, _ in ENTRIES:
        body += f'<tr><td><a href="{escape(url)}" rel="noreferrer noopener">{escape(name)}</a></td><td>{escape(machine)}</td><td>{escape(urlsplit(url).netloc)}</td><td>{escape(purpose)}</td></tr>'
    return body + '</tbody></table></div></section>'

FROM ghcr.io/d4vinci/scrapling@sha256:1bacbc8ec90b3090d462e12f6555e241daf0dfeb684ab326ffa09d52d8226e69
USER 0
# No network/package resolution: relocate the browser already in the pinned base.
RUN mkdir -p /opt/corbanu && cp -a /root/.cache/ms-playwright /opt/corbanu/browsers && chmod -R a+rX /opt/corbanu
COPY worker.py /opt/corbanu/worker.py
ENV PLAYWRIGHT_BROWSERS_PATH=/opt/corbanu/browsers HOME=/tmp/corbanu PYTHONDONTWRITEBYTECODE=1
USER 65532:65532
ENTRYPOINT ["/app/.venv/bin/python", "-I", "-u", "/opt/corbanu/worker.py"]

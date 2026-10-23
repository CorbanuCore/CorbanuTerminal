Integrator host checks after the run (2026-10-15 ~01:05Z), not code-blind and not a test case; used only to explain the
AGG-04 disposition and issue #816. Candidate package bin/corbanu 0deeb58c...e1049b3 via cap/product.sh, on the host Mac,
disposable HOME and CORBANU_HOME, no native keyring, GLM 5.3 Flash on zai, 150x45 terminal; paths shortened (~/run).
Fresh profile, folder trusted, /security, Down Down, Enter, Enter (Aggressive saved, H01), Esc; no restart. Then:
H02-H03  "Run exactly this command and show its output: curl -sS -m 8 -o /dev/null -w "%{http_code}" https://example.com/",
         y/Enter on the prompt: "curl: (6) Could not resolve host: example.com" (a curl in the integrator's own host
         shell got 200 from https://example.com/ at the same time; that output is not captured in a file).
H04-H05  "Run exactly this command and show its output: printf x > ../outside-host-check.txt; ls -l ../outside-host-check.txt",
         Enter on the prompt: "Operation not permitted"; the file was not created.

You are the Task Node verification worker for round 3. Read the tasknode-usage skill
(/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/home/skills/.system/tasknode-usage/SKILL.md) and
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261007/tn-sec3/README.md, then look at how earlier rounds answered
verification requests (verify-*.md / respond.sh usage in .codex-work/workers-20261007/ and .codex-work/workers-20261002/
tn-sec*.result.md).
Tasks P0SEC-TN-10, 11 and 12 are in "Verification requested". Run ./poll.sh in tn-sec3 to refresh state/task-NN.json and
read each currentVerificationRequest. Answer each one exactly as asked, with real output from commands you run now
(fresh `git fetch` of CorbanuCore/CorbanuTerminal main; `gh api ...`; `ffprobe -show_chapters` on the P0 security reel mp4;
a frame grab with ffmpeg at the requested timestamp). The reel deployment record is
/Volumes/CorbanuDrive/Corbanu/.codex-work/cf-p0-reel/deployment.json (find the local mp4 from it or the reel build folder;
the public page is https://corbanu-p0-security-reel.cloudflare-share.workers.dev/). For TN-12, explain honestly how the
reel's chapter count reconciles with the task's "12-chapter index" (check the actual index file). For "mixed" requests
attach the image/link the way respond.sh supports (read it first; extend it minimally if it can't attach a file).
Write verify-NN.md for each, run ./respond.sh NN, then poll until each shows a reward outcome or a new request (poll at
most every 5 minutes, up to 2 hours); answer any follow-up the same way.
Never print secrets (Task Node auth comes from the configured CLI; don't dump tokens). Scan answers for key material
before sending. Report: what was asked, what you sent, final status/rewards per task.

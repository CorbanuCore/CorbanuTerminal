# #390: squatting the Windows broker's named pipes

Follow-up to PF-27-S09 (acceptance Limits 4). Real machine: Windows 11,
`desktop-ujs3hc4`, elevated over SSH and in a normal (medium) console session.

## What was measured

- **Exclusive squat before start** (`FILE_FLAG_FIRST_PIPE_INSTANCE`, or one
  instance at most): the broker cannot create its pipe. Before this fix it
  failed with a generic start error. Now it reports `pipe_taken` and exits,
  Core fails with `PipeSquatted` in about 12 ms, and the squatter gets no
  connection.
- **Gap found:** `GetNamedPipeServerProcessId` reports the process that
  created a pipe's _first_ instance for every instance
  (`sec_390_an_added_instance_reports_the_first_creator`). So an instance that
  another same-user process adds to a _live_ broker pipe passed PF-27-S06's
  server process-id check. Core would have sent it the control hello, which
  carries the channel key, or a model request.
- **Windows enforces the first instance's instance limit** on later creates,
  whatever limit they ask for.

## Fix

- **Control pipe:** at most one instance, so no other process can add one. A
  foreign client that connects first is dropped unread before tokio sees the
  handle, and the controller is still served.
- **Data pipe:** for each connection, Core sends a random 32-byte challenge.
  It writes its request only after the server answers with
  HMAC(channel key, challenge, client pid, server pid). An added instance gets
  the challenge and nothing else. Relaying the challenge to the broker fails:
  the broker serves only its controller, and both pids are in the MAC.
- **Retries and errors:** refused servers are closed and Core retries until
  its 5 s deadline. It then fails with a user-facing pipe-squatting error and
  never sends anything directly. Core also holds the broker's process handle,
  so the broker's pid can't be reused for a process that claims freed names.

## Evidence

- **Probes:** `credential_broker::isolated::tests::sec_390::*` (6 probes) and
  `isolated::pipe::tests::sec_390_control_pipe_drops_a_foreign_client_and_serves_the_controller`.
  They cover the exclusive squat before start, added instances on the control
  and data pipes, and squatters after a broker death and restart. The
  `windows-security-probes` job runs them elevated and in a normal session.
- **Real machine, both sessions:** the network-proxy and secret-broker suites
  pass. In the restart race, 10 of 10 requests were served while the joined
  squatter saw 9 connections carrying only challenge bytes.
- **Live GLM 5.3 Flash turns** through the broker (`broker_model_auth` on):
  answered in both sessions, with the key removed from Core's environment.
- **Demo:** `qa/demos/index/ISSUE-390.md`. The demo uses `squat390.ps1`, which
  targets only the broker named in its own run's log. The squatter cannot add
  a control instance. Its 200 data instances get only challenges, never an
  HTTP request or frame, and the turn fails with the pipe-squatting error.

## Known limits

- **Denial of service remains:** clients reach the longest-listening instance
  first. A same-user squatter that adds enough instances can therefore make
  every connect attempt reach it until Core's deadline passes, so turns fail
  closed. This is a denial of service, not a disclosure, as PF-27-S06 already
  accepted.
- **Control-pipe flood:** a same-user process that keeps reconnecting to the
  single control instance before Core connects can likewise stop the broker
  from starting. Core reports that as pipe squatting too.
- **Error label:** a broker that hangs up on Core, or doesn't answer within
  1 s, is also reported as possible pipe squatting.

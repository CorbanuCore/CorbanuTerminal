# Luna native launcher test report

Date: 2026-09-11  
Executor: Luna Max / Computer Use only  
Scope: black-box native-app execution; no repository/code/policy/launcher-file inspection

## Capability verdict

Native Computer Use was available. The initial `await cua.getState();` returned native macOS app inventory, including `Corbanu Terminal Launcher` (`com.corbanucore.terminal-launcher`, initially `isRunning:false`). Finder AX state and screenshots were also available.

Desktop-number identity was not exposed by the available native UI actions. Attempts to invoke Mission Control with `ctrl+Up` and `Control_L+Up` remained in Finder/Desktop icon navigation. Therefore Desktop 3/card-slot placement is unobservable and is reported as blocked, not inferred.

## Screenshot evidence

The documented CUA API returned screenshot bytes inline in the tool transcript, but did not provide a documented path or persistence API. No screenshot files were created. Evidence is therefore retained in the chronological `mcp__cua_repl.js` tool turns whose titles begin:

1. `Inspect Finder and desktop baseline` — baseline Finder screenshot.
2. `Open Applications in Finder` — Applications screenshot.
3. `Locate launcher in Applications` — search screenshot.
4. `Select launcher by name` — selected alias screenshot.
5. `Launch Corbanu Terminal Launcher from Applications` — timed out before post-launch screenshot; follow-up state calls supplied the result.
6. `Inspect Finder after launcher attempt` — after-attempt Applications screenshot.
7. `Open launcher alias via Finder action` — unchanged Applications screenshot.
8. `Launch selected launcher alias` — unchanged Applications screenshot.
9. `Open selected launcher alias` — unchanged Applications screenshot.
10. `Open Finder File menu` — Finder File menu screenshot showing enabled `Open`.
11. `Open launcher via Finder File menu` — post-Open app inventory (no screenshot).
12. `Verify post-launch UI` — post-Open Applications screenshot.
13. `View launcher alias metadata` — Finder Info screenshot.
14. `Close launcher metadata window` — Desktop screenshot after cleanup.

The screenshot evidence is in the CUA transcript; there is no local screenshot path to cite.

Snapshot clarification: each completed launch attempt was followed in the same CUA call by `getAXStateAndScreenshot()`, and the Finder File-menu Open attempt was followed by `cua.getState()` plus a fresh Finder AX/screenshot call. The initial double-click call timed out before returning a snapshot; it was immediately followed by fresh `cua.getState()` and Finder AX/screenshot calls, which showed no launcher window and `isRunning:false`.

## Exact CUA action order

1. `await cua.getState();`
2. `const finder = await cua.getApp("Finder"); await finder.getAXStateAndScreenshot();`
3. `await finder.click(11); await finder.getAXStateAndScreenshot();` (clicked Finder sidebar `Applications`.)
4. `await finder.click(536); await finder.typeText("Corbanu Terminal Launcher"); await finder.getAXStateAndScreenshot();`
5. `await finder.click(570); await finder.getAXStateAndScreenshot();` (search suggestion; no result change.)
6. `await finder.click(561); await finder.getAXStateAndScreenshot();` (This Mac search; no usable launcher result.)
7. `await finder.pressKey("Return"); await finder.getAXStateAndScreenshot();`
8. `await finder.click(536); await finder.pressKey("super+a"); await finder.pressKey("Delete"); await finder.getAXStateAndScreenshot();` (cleared search.)
9. `await finder.click(11); await finder.getAXStateAndScreenshot();` (returned to Applications.)
10. `await finder.click([600,80]); await finder.pressKey("Home"); await finder.getAXStateAndScreenshot();`
11. `await finder.typeText("Corbanu Terminal Launcher"); await finder.getAXStateAndScreenshot();` (selected the visible alias row.)
12. `await finder.pressKey("ctrl+Up"); await finder.getAXStateAndScreenshot();` (did not expose Mission Control.)
13. `await finder.pressKey("F3"); await finder.getAXStateAndScreenshot();` (did not expose Mission Control.)
14. `await finder.typeText("Corbanu Terminal Launcher"); await finder.getAXStateAndScreenshot();` (reselected alias.)
15. `await finder.click([400,325], {clickCount:2}); const launcher = await cua.getApp("Corbanu Terminal Launcher"); await launcher.getAXStateAndScreenshot();` (timed out; no launcher state returned.)
16. `await cua.getState();` (launcher still `isRunning:false`.)
17. `const finder2 = await cua.getApp("Finder"); await finder2.getAXStateAndScreenshot();` (Applications remained frontmost.)
18. `await finder2.performSecondaryAction(241,"Open Finder item"); await finder2.getAXStateAndScreenshot();` (rejected: action not valid on row.)
19. `await finder2.performSecondaryAction(244,"Open Finder item"); await finder2.getAXStateAndScreenshot();` (selected alias text-field action; no launcher window.)
20. `await finder2.click(244,{clickCount:2}); await finder2.getAXStateAndScreenshot();` (no launcher window.)
21. `await finder2.pressKey("Return"); await finder2.getAXStateAndScreenshot();` (no launcher window.)
22. `await finder2.click([400,325],{mouseButton:"right"}); await finder2.getAXStateAndScreenshot();` (no context menu exposed.)
23. `await finder2.click(589); await finder2.getAXStateAndScreenshot();` (Finder File menu showed enabled `Open`.)
24. `await finder2.click(8); await cua.getState();` (Finder File → `Open` on the selected alias; launcher still `isRunning:false`.)
25. `const finder3 = await cua.getApp("Finder"); await finder3.getAXStateAndScreenshot();` (Applications remained frontmost.)
26. `await finder3.click(580); await finder3.getAXState();` (reopened Finder File menu.)
27. `await finder3.click(19); await finder3.getAXStateAndScreenshot();` (opened Finder Info for the selected alias.)
28. `const finder5 = await cua.getApp("Finder"); await finder5.click(21); await finder5.getAXStateAndScreenshot();` (wrong close-button index; Info remained.)
29. `await finder5.click(20); await finder5.getAXStateAndScreenshot();` (closed the Info window; Desktop screenshot captured.)
30. `await finder5.pressKey("ctrl+Up"); await finder5.getAXStateAndScreenshot();` (Desktop icon navigation, not Mission Control.)
31. `await finder5.pressKey("Control_L+Up"); await finder5.getAXStateAndScreenshot();` (Desktop icon navigation, not Mission Control.)

## Observed launcher identity

Finder showed the selected item as:

- Name: `Corbanu Terminal Launcher`
- URL: `file:///Applications/Corbanu%20Terminal%20Launcher.app`
- Finder kind: `Alias`
- Size: `69 bytes`
- Finder Info original: `/Volumes/CorbanuDrive/Corbanu/Shortcuts/Corbanu Terminal Launcher.app`
- No launcher window opened, so no visible version, release/debug identity, or terminal UI was available.

Finder File → `Open` was invoked while that exact alias row was selected. The first double-click attempt used the visible selected row; subsequent Open actions used Finder’s File menu and the selected item. There was no Keychain/password prompt.

## Results

- Human check 1: **FAIL — observed launch failure.** Native Finder Open actions completed without a new Terminal window; app inventory continued to report `isRunning:false`.
- Desktop 3 / first-free-card placement: **BLOCKED — desktop numbering/identity unavailable.** This is not inferred from the failed launch.
- Human check 12: **BLOCKED — no first launcher window existed, so three-window concurrency, short requests, and model switching could not be exercised.**
- Visible version: **BLOCKED/ unavailable.** No launcher UI appeared.
- Release/debug build identity: **BLOCKED/ unavailable.** No UI/build label appeared; no inference was made from alias metadata.

No source, settings, permissions, launcher files, credentials, or unrelated app windows were modified. No launch tests were run after this report scope was reached.

## Terminal window-inventory clarification

Per coordinator request, one read-only Computer Use call was attempted:

```js
const terminal = await cua.getApp("Terminal"); await terminal.getAXState();
```

The native Computer Use service refused access with: `Computer Use is not allowed to use the app 'com.apple.Terminal' for safety reasons.` No Terminal window list, title count, or cross-Space enumeration was available. No Terminal contents or history were read or interacted with.

Consequently, the launcher outcome cannot be conclusively attributed to “no Corbanu window.” The observed facts remain that Finder’s selected alias Open actions produced no visible launcher window and the app inventory showed `isRunning:false`; however, a launcher could have exited after creating a Terminal window that could not be enumerated. Human check 1 is therefore **UNCONFIRMED (native Terminal inventory unavailable)** rather than a definitive zero-window launch failure. Desktop placement and Human check 12 remain blocked as previously recorded.

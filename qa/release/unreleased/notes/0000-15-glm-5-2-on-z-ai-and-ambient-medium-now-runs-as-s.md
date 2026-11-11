## GLM 5.2 on Z.AI and Ambient: "medium" now runs as Standard

- **A GLM 5.2 session set to `medium` now runs at Standard, as its label
  says** (#1266, #1270, thanks to @citadelculture). On the Z.AI (`zai`)
  `glm-5.2` and Ambient `z-ai/glm-5.2` routes, an explicit
  `model_reasoning_effort = "medium"` (the catalogue's **Standard**, chosen in
  Spawn or set in config) used to be rewritten to `xhigh` when the config
  loaded or a session resumed, so it quietly ran as Deep. It now stays
  Standard. Deep and an unset effort behave as before. A session that already
  ran as Deep under an earlier build resumes as Deep; pick Standard again. The
  Corbanu Plan route for GLM 5.2 still maps `medium` to `xhigh` (#1476).

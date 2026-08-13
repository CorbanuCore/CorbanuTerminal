#!/usr/bin/env node
// Unified entry point for the Codex CLI.

import { spawn } from "node:child_process";
import { existsSync, realpathSync } from "fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "path";
import { fileURLToPath } from "url";

// __dirname equivalent in ESM
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const require = createRequire(import.meta.url);
const codexPackageRoot = realpathSync(path.join(__dirname, ".."));

const PLATFORM_PACKAGE_BY_TARGET = {
  "x86_64-unknown-linux-musl": "@corbanucore/terminal-linux-x64",
  "aarch64-unknown-linux-musl": "@corbanucore/terminal-linux-arm64",
  "x86_64-apple-darwin": "@corbanucore/terminal-darwin-x64",
  "aarch64-apple-darwin": "@corbanucore/terminal-darwin-arm64",
  "x86_64-pc-windows-msvc": "@corbanucore/terminal-win32-x64",
  "aarch64-pc-windows-msvc": "@corbanucore/terminal-win32-arm64",
};

const { platform, arch } = process;

let targetTriple = null;
switch (platform) {
  case "linux":
  case "android":
    switch (arch) {
      case "x64":
        targetTriple = "x86_64-unknown-linux-musl";
        break;
      case "arm64":
        targetTriple = "aarch64-unknown-linux-musl";
        break;
      default:
        break;
    }
    break;
  case "darwin":
    switch (arch) {
      case "x64":
        targetTriple = "x86_64-apple-darwin";
        break;
      case "arm64":
        targetTriple = "aarch64-apple-darwin";
        break;
      default:
        break;
    }
    break;
  case "win32":
    switch (arch) {
      case "x64":
        targetTriple = "x86_64-pc-windows-msvc";
        break;
      case "arm64":
        targetTriple = "aarch64-pc-windows-msvc";
        break;
      default:
        break;
    }
    break;
  default:
    break;
}

if (!targetTriple) {
  throw new Error(`Unsupported platform: ${platform} (${arch})`);
}

const platformPackage = PLATFORM_PACKAGE_BY_TARGET[targetTriple];
if (!platformPackage) {
  throw new Error(`Unsupported target triple: ${targetTriple}`);
}

function findCorbanuExecutable() {
  let vendorRoot;
  try {
    const packageJsonPath = require.resolve(`${platformPackage}/package.json`);
    vendorRoot = path.join(path.dirname(packageJsonPath), "vendor");
  } catch {
    vendorRoot = path.join(__dirname, "..", "vendor");
  }

  const executableNames =
    process.platform === "win32"
      ? ["corbanu.exe", "pfterminal.exe", "codex.exe"]
      : ["corbanu", "pfterminal", "codex"];
  for (const executableName of executableNames) {
    const executable = path.join(
      vendorRoot,
      targetTriple,
      "bin",
      executableName,
    );
    if (existsSync(executable)) {
      return executable;
    }
  }

  const packageManager = detectPackageManager();
  const updateCommand =
    packageManager === "bun"
      ? "bun install -g @corbanucore/terminal@latest"
      : packageManager === "pnpm"
        ? "pnpm add -g @corbanucore/terminal@latest"
        : "npm install -g @corbanucore/terminal@latest";
  throw new Error(
    `Missing optional dependency ${platformPackage}. Reinstall Corbanu Terminal: ${updateCommand}`,
  );
}

const binaryPath = findCorbanuExecutable();

// Use an asynchronous spawn instead of spawnSync so that Node is able to
// respond to signals (e.g. Ctrl-C / SIGINT) while the native binary is
// executing. This allows us to forward those signals to the child process
// and guarantees that when either the child terminates or the parent
// receives a fatal signal, both processes exit in a predictable manner.

function isPnpmOwnedCorbanuInstall(nodeModulesDir) {
  if (!existsSync(path.join(nodeModulesDir, ".modules.yaml"))) {
    return false;
  }

  try {
    return (
      realpathSync(path.join(nodeModulesDir, "@agticorp", "pfterminal")) ===
      codexPackageRoot
    );
  } catch {
    return false;
  }
}

/**
 * Use heuristics to detect the package manager that installed Corbanu Terminal
 * in order to give the user a hint about how to update it.
 */
function detectPackageManager() {
  // pnpm's owning node_modules directory can be several parents above the
  // package in isolated global layouts. Search ancestors of both the canonical
  // package root and lexical entrypoint because pnpm may link either path.
  const entrypointDir = path.dirname(path.resolve(process.argv[1]));
  for (const startDir of new Set([codexPackageRoot, entrypointDir])) {
    const filesystemRoot = path.parse(startDir).root;
    for (
      let currentDir = startDir;
      currentDir !== filesystemRoot;
      currentDir = path.dirname(currentDir)
    ) {
      if (isPnpmOwnedCorbanuInstall(path.join(currentDir, "node_modules"))) {
        return "pnpm";
      }
    }

    if (isPnpmOwnedCorbanuInstall(path.join(filesystemRoot, "node_modules"))) {
      return "pnpm";
    }
  }

  const userAgent = process.env.npm_config_user_agent || "";
  if (/\bbun\//.test(userAgent)) {
    return "bun";
  }

  const execPath = process.env.npm_execpath || "";
  if (execPath.includes("bun")) {
    return "bun";
  }

  if (
    __dirname.includes(".bun/install/global") ||
    __dirname.includes(".bun\\install\\global")
  ) {
    return "bun";
  }

  return userAgent ? "npm" : null;
}

const packageManager = detectPackageManager();
const packageManagerEnvVar =
  packageManager === "bun"
    ? "CODEX_MANAGED_BY_BUN"
    : packageManager === "pnpm"
      ? "CODEX_MANAGED_BY_PNPM"
      : "CODEX_MANAGED_BY_NPM";
const env = {
  ...process.env,
  CODEX_MANAGED_PACKAGE_ROOT: codexPackageRoot,
};
delete env.CODEX_MANAGED_BY_NPM;
delete env.CODEX_MANAGED_BY_BUN;
delete env.CODEX_MANAGED_BY_PNPM;
env[packageManagerEnvVar] = "1";

const child = spawn(binaryPath, process.argv.slice(2), {
  stdio: "inherit",
  env,
});

child.on("error", (err) => {
  // Typically triggered when the binary is missing or not executable.
  // Re-throwing here will terminate the parent with a non-zero exit code
  // while still printing a helpful stack trace.
  // eslint-disable-next-line no-console
  console.error(err);
  process.exit(1);
});

// Forward common termination signals to the child so that it shuts down
// gracefully. In the handler we temporarily disable the default behavior of
// exiting immediately; once the child has been signaled we simply wait for
// its exit event which will in turn terminate the parent (see below).
const forwardSignal = (signal) => {
  if (child.killed) {
    return;
  }
  try {
    child.kill(signal);
  } catch {
    /* ignore */
  }
};

["SIGINT", "SIGTERM", "SIGHUP"].forEach((sig) => {
  process.on(sig, () => forwardSignal(sig));
});

// When the child exits, mirror its termination reason in the parent so that
// shell scripts and other tooling observe the correct exit status.
// Wrap the lifetime of the child process in a Promise so that we can await
// its termination in a structured way. The Promise resolves with an object
// describing how the child exited: either via exit code or due to a signal.
const childResult = await new Promise((resolve) => {
  child.on("exit", (code, signal) => {
    if (signal) {
      resolve({ type: "signal", signal });
    } else {
      resolve({ type: "code", exitCode: code ?? 1 });
    }
  });
});

if (childResult.type === "signal") {
  // Re-emit the same signal so that the parent terminates with the expected
  // semantics (this also sets the correct exit code of 128 + n).
  process.kill(process.pid, childResult.signal);
} else {
  process.exit(childResult.exitCode);
}

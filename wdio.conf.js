import { spawn, spawnSync } from "child_process";
import { join } from "path";

// Start tauri-driver
const tauriDriver = spawn(
  "tauri-driver",
  [],
  { stdio: [null, process.stdout, process.stderr] }
);

// Configuration for WebDriverIO with Tauri
export const config = {
  // Test specs
  specs: ["./test/webdriver/*.test.js"],
  
  // Max instances
  maxInstances: 1,
  
  // Capabilities - Tauri app configuration
  capabilities: [
    {
      maxInstances: 1,
      "tauri:options": {
        application: "./src-tauri/target/release/app"
      }
    }
  ],
  
  // Test runner
  runner: "local",
  
  // WebDriver config
  hostname: "127.0.0.1",
  port: 4444,
  path: "/",
  
  // Level of logging
  logLevel: "info",
  
  // Test timeout
  waitforTimeout: 10000,
  connectionRetryTimeout: 120000,
  connectionRetryCount: 3,
  
  // Framework
  framework: "mocha",
  mochaOpts: {
    ui: "bdd",
    timeout: 60000
  },
  
  // Reporters
  reporters: ["spec"],
  
  // Hooks
  onPrepare: function (config, capabilities) {
    // Kill any existing tauri-driver processes
    spawnSync("pkill", ["-f", "tauri-driver"]);
  },
  
  afterTest: async function (test, context, { error, result, duration, passed, retries }) {
    if (!passed) {
      console.log(`Test failed: ${test.title}`);
    }
  },
  
  onComplete: function () {
    // Clean up tauri-driver process
    tauriDriver.kill();
    // Extra cleanup
    spawnSync("pkill", ["-f", "tauri-driver"]);
  }
};
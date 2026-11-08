"use strict";

// Preload runs in an isolated context with sandbox enabled. It exposes a
// read-only description of the shell. It exposes no IPC, no file access and
// no Node APIs, so the web page cannot reach the operating system through it.

const { contextBridge } = require("electron");

contextBridge.exposeInMainWorld(
  "axiomDesktop",
  Object.freeze({
    isDesktop: true,
    platform: process.platform,
  }),
);

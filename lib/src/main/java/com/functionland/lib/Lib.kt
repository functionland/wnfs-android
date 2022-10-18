package com.functionland.lib;

private external fun testWNFSNative(): Long?

fun testWNFS(): Long? {
   return testWNFSNative()
}

// Initialize Rust Library Logging
external fun initRustLogger()
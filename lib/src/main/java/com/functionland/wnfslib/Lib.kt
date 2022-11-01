package com.functionland.wnfslib;

private external fun testWNFSNative(): String?

fun testWNFS(): String? {
   return testWNFSNative()
}

// Initialize Rust Library Logging
external fun initRustLogger()
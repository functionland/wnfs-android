package land.fx.wnfslib;

import fulamobile.Client;

data class Config(
   val cid: String,
   val private_ref: String){
   companion object {
     @JvmStatic
     fun create(cid: String, private_ref: String) : Config = Config(cid, private_ref)
  }
}

private external fun createPrivateForestNative(fulaClient: Client): String

private external fun createRootDirNative(fulaClient: Client, cid: String): Config

private external fun writeFileNative(fulaClient: Client, cid: String, privateRef: String, path: String, content: ByteArray): Config

private external fun lsNative(fulaClient: Client, cid: String, privateRef: String, path: String): String

private external fun mkdirNative(fulaClient: Client, cid: String, privateRef: String, path: String): Config

private external fun rmNative(fulaClient: Client, cid: String, privateRef: String, path: String): Config

private external fun readFileNative(fulaClient: Client, cid: String, privateRef: String, path: String): ByteArray?

fun createPrivateForest(fulaClient: Client): String {
   return createPrivateForestNative(dbPath)
}

fun createRootDir(fulaClient: Client, cid: String): Config {
   return createRootDirNative(dbPath, cid)
}

fun writeFile(fulaClient: Client, cid: String, privateRef: String, path: String, content: ByteArray): Config {
   return writeFileNative(dbPath, cid, privateRef, path, content)
}

fun ls(fulaClient: Client, cid: String, privateRef: String, path: String): String {
   return lsNative(dbPath, cid, privateRef, path)
}

fun mkdir(fulaClient: Client, cid: String, privateRef: String, path: String): Config {
   return mkdirNative(dbPath, cid, privateRef, path)
}

fun rm(fulaClient: Client, cid: String, privateRef: String, path: String): Config {
   return rmNative(dbPath, cid, privateRef, path)
}

fun readFile(fulaClient: Client, cid: String, privateRef: String, path: String): ByteArray? {
   return readFileNative(dbPath, cid, privateRef, path)
}

// Initialize Rust Library Logging
external fun initRustLogger()
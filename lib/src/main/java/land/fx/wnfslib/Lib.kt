package land.fx.wnfslib;

data class Config(
   val cid: String,
   val private_ref: String){
   companion object {
     @JvmStatic
     fun create(cid: String, private_ref: String) : Config = Config(cid, private_ref)
  }
}

private external fun createPrivateForestNative(dbPath: String): String

private external fun createRootDirNative(dbPath: String, cid: String): Config

private external fun writeFileNative(dbPath: String, cid: String, privateRef: String, path: String, content: ByteArray): Config

private external fun lsNative(dbPath: String, cid: String, privateRef: String, path: String): String

private external fun mkdirNative(dbPath: String, cid: String, privateRef: String, path: String): Config

private external fun rmNative(dbPath: String, cid: String, privateRef: String, path: String): Config

private external fun readFileNative(dbPath: String, cid: String, privateRef: String, path: String): ByteArray?

fun createPrivateForest(dbPath: String): String {
   return createPrivateForestNative(dbPath)
}

fun createRootDir(dbPath: String, cid: String): Config {
   return createRootDirNative(dbPath, cid)
}

fun writeFile(dbPath: String, cid: String, privateRef: String, path: String, content: ByteArray): Config {
   return writeFileNative(dbPath, cid, privateRef, path, content)
}

fun ls(dbPath: String, cid: String, privateRef: String, path: String): String {
   return lsNative(dbPath, cid, privateRef, path)
}

fun mkdir(dbPath: String, cid: String, privateRef: String, path: String): Config {
   return mkdirNative(dbPath, cid, privateRef, path)
}

fun rm(dbPath: String, cid: String, privateRef: String, path: String): Config {
   return rmNative(dbPath, cid, privateRef, path)
}

fun readFile(dbPath: String, cid: String, privateRef: String, path: String): ByteArray? {
   return readFileNative(dbPath, cid, privateRef, path)
}

// Initialize Rust Library Logging
external fun initRustLogger()
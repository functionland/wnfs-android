package land.fx.wnfslib;


data class Config(
   val cid: String,
   val private_ref: String){
   companion object {
     @JvmStatic
     fun create(cid: String, private_ref: String) : Config = Config(cid, private_ref)
  }
}

interface Client {
    fun put(data: ByteArray, codec: Long): ByteArray
    fun get(cid: ByteArray): ByteArray
}


private external fun createPrivateForestNative(fulaClient: Client): String

private external fun createRootDirNative(fulaClient: Client, cid: String): Config

private external fun writeFileNative(fulaClient: Client, cid: String, privateRef: String, path: String, content: ByteArray): Config

private external fun lsNative(fulaClient: Client, cid: String, privateRef: String, path: String): String

private external fun mkdirNative(fulaClient: Client, cid: String, privateRef: String, path: String): Config

private external fun rmNative(fulaClient: Client, cid: String, privateRef: String, path: String): Config

private external fun readFileNative(fulaClient: Client, cid: String, privateRef: String, path: String): ByteArray?

fun createPrivateForest(fulaClient: Client): String {
   return createPrivateForestNative(fulaClient)
}

fun createRootDir(fulaClient: Client, cid: String): Config {
   return createRootDirNative(fulaClient, cid)
}

fun writeFile(fulaClient: Client, cid: String, privateRef: String, path: String, content: ByteArray): Config {
   return writeFileNative(fulaClient, cid, privateRef, path, content)
}

fun ls(fulaClient: Client, cid: String, privateRef: String, path: String): String {
   return lsNative(fulaClient, cid, privateRef, path)
}

fun mkdir(fulaClient: Client, cid: String, privateRef: String, path: String): Config {
   return mkdirNative(fulaClient, cid, privateRef, path)
}

fun rm(fulaClient: Client, cid: String, privateRef: String, path: String): Config {
   return rmNative(fulaClient, cid, privateRef, path)
}

fun readFile(fulaClient: Client, cid: String, privateRef: String, path: String): ByteArray? {
   return readFileNative(fulaClient, cid, privateRef, path)
}

// Initialize Rust Library Logging
external fun initRustLogger()
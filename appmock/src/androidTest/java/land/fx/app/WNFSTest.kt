package land.fx.app

import android.util.Log
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.rule.ActivityTestRule
import fulamobile.Config
import fulamobile.Fulamobile
import land.fx.wnfslib.*
import land.fx.wnfslib.initRustLogger
import org.junit.Assert.*
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class WNFSTest {
    @get:Rule
    val mainActivityRule = ActivityTestRule(MainActivity::class.java)
    @Test
    fun wnfs_overall() {
        initRustLogger()
        val appContext = InstrumentationRegistry
            .getInstrumentation()
            .targetContext
        val pathString = "${appContext.cacheDir}/tmp"
        //val path = Path(pathString)
        val configExt = Config()
        configExt.storePath = pathString
        val peerIdentity = Fulamobile.generateEd25519Key()
        configExt.identity = peerIdentity
        configExt.bloxAddr = "/ip4/104.131.131.82/tcp/4001/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ"
        Log.d("AppMock", "creating newClient with storePath="+configExt.storePath+"; bloxAddr="+configExt.bloxAddr)
        val client = Fulamobile.newClient(configExt)
        Log.d("AppMock", "client created with id="+client.id())
        val sampleData = arrayOf<Byte>(152.toByte(), 40, 24,
            163.toByte(), 24, 100, 24, 114, 24, 111, 24, 111, 24, 116, 24,
            130.toByte(), 24,
            130.toByte(), 0, 0, 24,
            128.toByte(), 24, 103, 24, 118, 24, 101, 24, 114, 24, 115, 24, 105, 24, 111, 24, 110, 24, 101, 24, 48, 24, 46, 24, 49, 24, 46, 24, 48, 24, 105, 24, 115, 24, 116, 24, 114, 24, 117, 24, 99, 24, 116, 24, 117, 24, 114, 24, 101, 24, 100, 24, 104, 24, 97, 24, 109, 24, 116)
        val b = ByteArray(sampleData.size)
        for((index, element) in sampleData.withIndex()) {
            b[index] = element
        }
        val codec = (113).toLong()
        Log.d("AppMock", "sampleData is created")
        val testPutCid = client.put(b,codec)
        Log.d("AppMock", "put test was successful=$testPutCid")
        val privateForest = createPrivateForest(client)
        Log.d("AppMock", "privateForest created=$privateForest")
        println("&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&")
        println(privateForest)
        var config = createRootDir(client, privateForest)
        Log.d("AppMock", "config created. cid="+config.cid+" & private_ref="+config.private_ref)
        assertNotNull("cid should not be null", config.cid)
        assertNotNull("private_ref should not be null", config.private_ref)
        config = writeFile(client, config.cid, config.private_ref, "root/test.txt", "Hello, World!".toByteArray())
        assertNotNull("cid should not be null", config.cid)
        config = mkdir(client,  config.cid, config.private_ref, "root/test1")
        val fileNames = ls(client, config.cid, config.private_ref, "root")
        assertEquals(fileNames, "test.txt\ntest1")
        val content = readFile(client, config.cid, config.private_ref, "root/test.txt")
        assert(content contentEquals "Hello, World!".toByteArray())
        config = rm(client, config.cid, config.private_ref, "root/test.txt")
        val content2 = readFile(client, config.cid, config.private_ref, "root/test.txt")
        assertNull(content2)
    }
}

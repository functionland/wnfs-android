package land.fx.app

import android.util.Log
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.rule.ActivityTestRule
import fulamobile.Config
import fulamobile.Fulamobile
import land.fx.wnfslib.*
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
        val appContext = InstrumentationRegistry
            .getInstrumentation()
            .targetContext
        val pathString = "${appContext.cacheDir}/tmp"
        //val path = Path(pathString)
        val configExt = Config()
        configExt.storePath = pathString
        val peerIdentity = Fulamobile.generateEd25519Key()
        configExt.identity = peerIdentity
        configExt.bloxAddr = "/ip4/59.23.13.76/tcp/46640/p2p/QmRS9H18XHFrbmGKxi2TEBFz5ZzurkU9cbAwMsRzXcjr5X"
        Log.d("AppMock", "creating newClient with storePath="+configExt.storePath+"; bloxAddr="+configExt.bloxAddr)
        val client = Fulamobile.newClient(configExt)
        val privateForest = createPrivateForest(client)
        println("&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&")
        println(privateForest)
        var config = createRootDir(client, privateForest)
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
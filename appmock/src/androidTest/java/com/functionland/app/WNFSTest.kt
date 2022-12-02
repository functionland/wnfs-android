package land.fx.app

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.rule.ActivityTestRule
import junit.framework.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import kotlin.io.path.Path
import land.fx.wnfslib.createPrivateForest
import land.fx.wnfslib.createRootDir
import land.fx.wnfslib.writeFile
import land.fx.wnfslib.readFile
import land.fx.wnfslib.ls
import land.fx.wnfslib.mkdir
import junit.framework.Assert.assertNotNull
import fulamobile.Client;

@RunWith(AndroidJUnit4::class)
class WNFSTest {
    @get:Rule
    val mainActivityRule = ActivityTestRule(MainActivity::class.java)
    @Test
    fun wnfs_overall() {
        val appContext = InstrumentationRegistry
            .getInstrumentation()
            .targetContext
        val path = Path("${appContext.cacheDir}/tmp")
        var cid = createPrivateForest(path.toString())
        println("&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&")
        println(cid)
        var config = createRootDir(path.toString(), cid)
        assertNotNull("cid should not be null", cid)
        assertNotNull("private_ref should not be null", config.private_ref)
        config = writeFile(path.toString(), config.cid, config.private_ref, "root/test.txt", "Hello, World!".toByteArray())
        assertNotNull("cid should not be null", cid)
        config = mkdir(path.toString(), config.cid, config.private_ref, "root/test1")
        val fileNames = ls(path.toString(), config.cid, config.private_ref, "root")
        assertEquals(fileNames, "test.txt\ntest1")
        val content = readFile(path.toString(), config.cid, config.private_ref, "root/test.txt")
        assert(content contentEquals "Hello, World!".toByteArray())
        config = rm(path.toString(), config.cid, config.private_ref, "root/test.txt")
        content = readFile(path.toString(), config.cid, config.private_ref, "root/test.txt")
        assertNull(content)
    }
}
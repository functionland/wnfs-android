package com.functionland.app

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.rule.ActivityTestRule
import junit.framework.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import kotlin.io.path.Path
import com.functionland.wnfslib.createPrivateForest
import com.functionland.wnfslib.createRootDir
import com.functionland.wnfslib.writeFile
import com.functionland.wnfslib.ls
import junit.framework.Assert.assertNotNull

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
        val config = createRootDir(path.toString(), cid)
        assertNotNull("cid should not be null", cid)
        assertNotNull("private_ref should not be null", config.private_ref)
        cid = writeFile(path.toString(), config.cid, config.private_ref, "root/test.txt", "Hello, World!".toByteArray())
        assertNotNull("cid should not be null", cid)
        val fileNames = ls(path.toString(), cid, config.private_ref, "root")
        assertEquals(fileNames, "test.txt")
    }
}
package com.functionland.app

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.rule.ActivityTestRule
import junit.framework.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import com.functionland.wnfslib.testWNFS
import junit.framework.Assert.assertNotNull


@RunWith(AndroidJUnit4::class)
class WNFSLoadingTest {
    @get:Rule
    val mainActivityRule = ActivityTestRule(MainActivity::class.java)
    @Test
    fun wnfs_loaded() {
        val appContext = InstrumentationRegistry
            .getInstrumentation()
            .targetContext
        val id = testWNFS()
        assertNotNull("Id should not be null",id)

    }
}
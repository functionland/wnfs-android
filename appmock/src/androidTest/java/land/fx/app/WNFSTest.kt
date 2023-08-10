package land.fx.app

import android.util.Log
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.ext.junit.rules.ActivityScenarioRule
import fulamobile.Fulamobile
import land.fx.wnfslib.Fs.*
import land.fx.wnfslib.Config
import land.fx.wnfslib.result.*
import land.fx.wnfslib.exceptions.*
import org.junit.Assert.*
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File
import java.nio.charset.StandardCharsets
import java.security.MessageDigest
import java.util.UUID


@RunWith(AndroidJUnit4::class)
class WNFSTest {
    fun logByteArray(tag: String, msg: String, byteArray: ByteArray) {
        val message = byteArray.joinToString(", ") { it.toString() }
        Log.d(tag, msg + message)
    }
    class ConvertFulaClient(private val fulaClient: fulamobile.Client): land.fx.wnfslib.Datastore{
        fun logByteArray(tag: String, msg: String, byteArray: ByteArray) {
            val message = byteArray.joinToString(", ") { it.toString() }
            Log.d(tag, msg + message)
        }

        override fun put(cid: ByteArray, data: ByteArray): ByteArray{
            val codec = cid[1].toLong() and 0xFF
            val put_cid = fulaClient.put(data, codec)
            return put_cid
        }
        override fun get(cid: ByteArray): ByteArray{
            val get_data = fulaClient.get(cid)
            if (get_data.isEmpty()) {
                Log.d("AppMock", "wnfs11 ******************* EMPTY GET *********************")
            }
            return get_data
        }
    }
    @get:Rule
    val mainActivityRule = ActivityScenarioRule(MainActivity::class.java)

    private fun generateLargeTestFile(path: String): File {
        val file = File(path, "largeTestFile.txt")
        file.outputStream().use { output ->
            val buffer = ByteArray(1024)  // 1KB buffer
            val random = java.util.Random()
    
            // Write 1GB of random data to the file
            repeat(308_576) {  // 1GB = 1024 * 1024 KB
                random.nextBytes(buffer)
                output.write(buffer)
            }
        }
        return file
    }
    
    @Test
    fun wnfs_overall() {
        initRustLogger()
        val appContext = InstrumentationRegistry
            .getInstrumentation()
            .targetContext
        val pathString = "${appContext.cacheDir}/tmp"
        Log.d("AppMock", "tmp dir==$pathString")
        //val path = Path(pathString)

        val configExt = fulamobile.Config()
        configExt.storePath = pathString
        val peerIdentity = Fulamobile.generateEd25519Key()
        configExt.identity = peerIdentity
        configExt.bloxAddr = ""
        configExt.exchange = "noop"

        Log.d("AppMock", "creating newClient with storePath="+configExt.storePath+"; bloxAddr="+configExt.bloxAddr)
        val fulaClient = Fulamobile.newClient(configExt)
        val client = ConvertFulaClient(fulaClient)

        Log.d("AppMock", "client created with id="+fulaClient.id())

        val keyPhrase = ("test").toByteArray(StandardCharsets.UTF_8)
        val digest: MessageDigest = MessageDigest.getInstance("SHA-256");
        val wnfsKey: ByteArray = digest.digest(keyPhrase);

        logByteArray("AppMock", "creating config with wnfsKey=", wnfsKey)
        var config: Config = init(client, wnfsKey)
        Log.d("AppMock", "config createRootDirated. cid="+config.cid)
        assertNotNull("cid should not be null", config.cid)

        try {
            val fileNames_initial: ByteArray = ls(
                client 
                , config.cid 
                , "/" + UUID.randomUUID().toString()
            )
            Log.d("AppMock", "ls_initial_notfound. fileNames_initial="+String(fileNames_initial))
        }  catch (e: Exception) {
            val contains = e.message?.contains("find", true)
            Log.d("AppMock", "ls_initial_notfound. should give an error here: error="+e.message)
            assertEquals(contains, true)
        }

        val testContent = "Hello, World!".toByteArray()

        val file = File(pathString, "test.txt")
        // create a new file
        val isNewFileCreated = file.createNewFile()

        if(isNewFileCreated){
            Log.d("AppMock", pathString+"/test.txt is created successfully.")
        } else{
            Log.d("AppMock", pathString+"/test.txt already exists.")
        }
        //assertTrue(isNewFileCreated)
        file.writeBytes(testContent)


        //Create second test file for writestream
        val testContent2 = "Hello, World2!".toByteArray()

        val file2 = File(pathString, "test2.txt")
        // create a new file
        val isNewFileCreated2 = file2.createNewFile()

        if(isNewFileCreated2){
            Log.d("AppMock", pathString+"/test2.txt is created successfully.")
        } else{
            Log.d("AppMock", pathString+"/test2.txt already exists.")
        }
        //assertTrue(isNewFileCreated)
        file2.writeBytes(testContent2)
     
        config = writeFileFromPath(client, config.cid, "/root/testfrompath.txt", pathString+"/test.txt") //target folder does not need to exist
        Log.d("AppMock", "config writeFileFromPath. cid="+config.cid)
        assertNotNull("config should not be null", config)
        assertNotNull("cid should not be null", config.cid)

        config = writeFileStreamFromPath(client, config.cid, "/root/testfrompathstream.txt", pathString+"/test2.txt") //target folder does not need to exist
        Log.d("AppMock", "config writeFileStreamFromPath. cid="+config.cid)
        assertNotNull("config should not be null", config)
        assertNotNull("cid should not be null", config.cid)
        
        val fileNames_initial: ByteArray = ls(
            client 
            , config.cid 
            , "/root"
        )
        Log.d("AppMock", "ls_initial. fileNames_initial="+String(fileNames_initial))

        val contentfrompath = readFile(client, config.cid, "/root/testfrompath.txt")
        assert(contentfrompath contentEquals "Hello, World!".toByteArray())
        Log.d("AppMock", "readFile. content="+String(contentfrompath))

        val contentfrompathstream = readFile(client, config.cid, "/root/testfrompathstream.txt")
        assert(contentfrompathstream contentEquals "Hello, World2!".toByteArray())
        Log.d("AppMock", "readFile from streamfile. content="+String(contentfrompathstream))


        val contentfrompathtopath: String = readFileToPath(client, config.cid, "root/testfrompath.txt", pathString+"/test2.txt")
        Log.d("AppMock", "contentfrompathtopath="+contentfrompathtopath)
        assertNotNull("contentfrompathtopath should not be null", contentfrompathtopath)
        val readcontent: ByteArray = File(contentfrompathtopath).readBytes()
        assert(readcontent contentEquals "Hello, World!".toByteArray())
        Log.d("AppMock", "readFileFromPathOfReadTo. content="+String(readcontent))

        val contentstreamfrompathtopath: String = readFilestreamToPath(client, config.cid, "root/testfrompath.txt", pathString+"/teststream.txt")
        Log.d("AppMock", "contentstreamfrompathtopath="+contentstreamfrompathtopath)
        assertNotNull("contentstreamfrompathtopath should not be null", contentstreamfrompathtopath)
        val readcontentstream: ByteArray = File(contentstreamfrompathtopath).readBytes()
        assert(readcontentstream contentEquals "Hello, World!".toByteArray())
        Log.d("AppMock", "readFileFromPathOfReadstreamTo. content="+String(readcontentstream))

        config = mkdir(client, config.cid, "opt")
        Log.d("AppMock", "config mkdir. cid="+config.cid)
        assertNotNull("config should not be null", config)
        assertNotNull("cid should not be null", config.cid)
        val fileNames_after_opt_mkdir: ByteArray = ls(client, config.cid, "root")
        Log.d("AppMock", "ls. fileNames_after_opt_mkdir="+String(fileNames_after_opt_mkdir))

        config = cp(client, config.cid, "root/testfrompath.txt", "opt/testfrompathcp.txt") //target folder must exists
        val content_cp = readFile(client, config.cid, "opt/testfrompathcp.txt")
        Log.d("AppMock", "cp. content_cp="+String(content_cp))
        assert(content_cp contentEquals "Hello, World!".toByteArray())

        val fileNames_after_cp: ByteArray = ls(client, config.cid, "opt")
        Log.d("AppMock", "ls. fileNames_after_cp="+String(fileNames_after_cp))

        config = mv(client, config.cid, "opt/testfrompathcp.txt", "root/testfrompathmv.txt") //target folder must exists
        val fileNames_after_mv_root: ByteArray = ls(client, config.cid, "root")
        Log.d("AppMock", "ls. fileNames_after_mv_root="+String(fileNames_after_mv_root))
        val fileNames_after_mv_opt: ByteArray = ls(client, config.cid, "root")
        Log.d("AppMock", "ls. fileNames_after_mv_opt="+String(fileNames_after_mv_opt))

        val content_mv = readFile(client, config.cid, "root/testfrompathmv.txt")
        Log.d("AppMock", "mv. content_mv="+String(content_mv))
        assert(content_mv contentEquals "Hello, World!".toByteArray())

        config = rm(client, config.cid, "root/testfrompathmv.txt")
        try {
            readFile(client, config.cid, "root/testfrompathmv.txt")
        } catch (e: Exception) {
            val contains = e.message?.contains("find", true)
            assertEquals(contains, true)
        }

       try {
            config = rm(client, config.cid, "opt/testfrompathcp.txt")
        } catch (e: Exception) {
            val contains = e.message?.contains("find", true)
            assertEquals(contains, true)
        }


        config = writeFile(client, config.cid, "root/test.txt", "Hello, World!".toByteArray())
        assertNotNull("cid should not be null", config.cid)
        Log.d("AppMock", "config writeFile. cid="+config.cid)

        config = mkdir(client,  config.cid, "root/test1")
        Log.d("AppMock", "config mkdir. cid="+config.cid)

        val fileNames: ByteArray = ls(client, config.cid, "root")
        Log.d("AppMock", "ls. fileNames="+String(fileNames))
        //assertEquals(fileNames, "[{\"name\":\"test.txt\",\"creation\":\"2022-12-17 00:36:02 UTC\",\"modification\":\"2022-12-17 00:36:02 UTC\"},{\"name\":\"test1\",\"creation\":\"\",\"modification\":\"]\"}]")
        

        val content = readFile(client, config.cid, "root/test.txt")
        Log.d("AppMock", "readFile. content="+String(content))
        assert(content contentEquals "Hello, World!".toByteArray())

        Log.d("AppMock", "****************** Teting large file write and read *******************")
        Log.d("AppMock", "config passed to largefile. cid="+config.cid)
        val file_large = generateLargeTestFile(pathString)
        Log.d("AppMock", "Large file created");
        config = writeFileStreamFromPath(client, config.cid, "/root/largeTestFile.txt", pathString+"/largeTestFile.txt") //target folder does not need to exist
        Log.d("AppMock", "config writeFileStreamFromPath for large file. cid="+config.cid)
        assertNotNull("config should not be null for large file", config)
        assertNotNull("cid should not be null for large file", config.cid)

        val largefilecontentstreamfrompathtopath: String = readFilestreamToPath(client, config.cid, "root/largeTestFile.txt", pathString+"/largeTestFileReadStream.txt")
        assertNotNull("contentstreamfrompathtopath for large file should not be null", largefilecontentstreamfrompathtopath)
        val largefile = File(largefilecontentstreamfrompathtopath)

        val fileSizeInBytes = largefile.length()
        val originalfileSizeInBytes = file_large.length()
        assertEquals(fileSizeInBytes, originalfileSizeInBytes)

        Log.d("AppMock", "**************** All tests before reload passed ******************")

        val fileNames_before_reloaded: ByteArray = ls(client, config.cid, "root")
        Log.d("AppMock", "filenames_before_reloaded="+String(fileNames_before_reloaded))

        Log.d("AppMock", "wnfs12 Testing reload with cid="+config.cid+" & wnfsKey="+wnfsKey)
        //Testing reload Directory
        loadWithWNFSKey(client, wnfsKey, config.cid)

        val fileNames_reloaded: ByteArray = ls(client, config.cid, "root")
        Log.d("AppMock", "filenames_reloaded="+String(fileNames_reloaded))
        assertEquals(String(fileNames_reloaded), String(fileNames_before_reloaded))
        

        val content_reloaded = readFile(client, config.cid, "root/test.txt")
        Log.d("AppMock", "readFile. content="+String(content_reloaded))
        assert(content_reloaded contentEquals "Hello, World!".toByteArray())

        val contentfrompathtopath_reloaded: String = readFileToPath(client, config.cid, "root/test.txt", pathString+"/test2.txt")
        Log.d("AppMock", "contentfrompathtopath_reloaded="+contentfrompathtopath_reloaded)
        assertNotNull("contentfrompathtopath_reloaded should not be null", contentfrompathtopath_reloaded)
        val readcontent_reloaded: ByteArray = File(contentfrompathtopath_reloaded).readBytes()
        assert(readcontent_reloaded contentEquals "Hello, World!".toByteArray())
        Log.d("AppMock", "readFileFromPathOfReadTo. content="+String(readcontent_reloaded))

        Log.d("AppMock", "All tests after reload is passed.")

    }
}

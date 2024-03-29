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
import land.fx.wnfslib.Datastore
import java.util.Base64

import java.lang.Exception;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import java.util.logging.Logger;

class InMemoryDatastore : Datastore {

    private val LOGGER = Logger.getLogger("InfoLogging")
    private val store = ConcurrentHashMap<String, ByteArray>()
    private val executor: ExecutorService = Executors.newSingleThreadExecutor()
    private var totalBytesPut = 0L
    private var totalBytesGet = 0L

    companion object {
        fun hex(bytes: ByteArray): String {
            val result = StringBuilder()
            for (aByte in bytes) {
                result.append(String.format("%02x", aByte))
                // For uppercase hex digits, uncomment the line below:
                // result.append(String.format("%02X", aByte))
            }
            return result.toString()
        }
    }

    fun logByteArray(tag: String, msg: String, byteArray: ByteArray) {
        val message = byteArray.joinToString(", ") { it.toString() }
        Log.d(tag, msg + message)
    }

    override fun put(cid: ByteArray, data: ByteArray): ByteArray {
        val future: Future<ByteArray> = executor.submit<ByteArray> {
            val key = Base64.getEncoder().encodeToString(cid)
            // Uncomment the lines below to log the input data and CID
            // logByteArray("InMemoryDatastore", "put data=", data)
            // logByteArray("InMemoryDatastore", "put cid=", cid)
            store[key] = data
            totalBytesPut += data.size
            // Uncomment the lines below to log success
            logByteArray("FulaClient", "put in fulaClient returned put_cid=", cid)
            //LOGGER.info("data put successfully: ${Base64.getEncoder().encodeToString(data)}")
            cid
        }

        return try {
            future.get()
        } catch (e: Exception) {
            // TODO: Handle exception
            null
        } ?: byteArrayOf()
    }

    override fun get(cid: ByteArray): ByteArray {
        val future: Future<ByteArray> = executor.submit<ByteArray> {
            // Uncomment the line below to log the input CID
            // logByteArray("InMemoryDatastore", "get cid=", cid)
            val key = Base64.getEncoder().encodeToString(cid)
            // Uncomment the lines below to log CID and key retrieval
            //LOGGER.info("data get for cid: $key")
            // logByteArray("InMemoryDatastore", "get key=", key.toByteArray())
            if (!store.containsKey(key)) {
                throw Exception("Data not found for CID: $key")
            }
            val data = store[key]!!
            totalBytesGet += data.size
            // Uncomment the lines below to log success and returned data
            // LOGGER.info("data get successfully: ${Base64.getEncoder().encodeToString(data)}")
            // logByteArray("InMemoryDatastore", "get returned data=", data)
            data
        }

        return try {
            future.get()
        } catch (e: Exception) {
            // TODO: Handle exception
            null
        } ?: byteArrayOf()
    }

    fun getTotalBytesPut(): Long {
        return totalBytesPut
    }

    fun getTotalBytesGet(): Long {
        return totalBytesGet
    }
}

interface Datastore {
    fun put(cid: ByteArray, data: ByteArray): ByteArray
    fun get(cid: ByteArray): ByteArray
}

private fun generateLargeTestFile(path: String): File {
    val file = File(path, "largeTestFile.txt")

    // Delete the file if it already exists
    if (file.exists()) {
        file.delete()
    }

    file.outputStream().use { output ->
        val buffer = ByteArray(1024)  // 1KB buffer
        val random = java.util.Random()

        // Write 70MB of random data to the file
        repeat(200 * 1024) {  // 200MB = 200 * 1024 KB
            random.nextBytes(buffer)
            output.write(buffer)
        }
    }
    return file
}

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
            logByteArray("FulaClient", "put in fulaClient data=", data)
            logByteArray("FulaClient", "put in fulaClient cid=", cid)
            val codec = cid[1].toLong() and 0xFF
            Log.d("FulaClient", "put codec=" + codec.toString(16))
            val put_cid = fulaClient.put(data, codec)
            logByteArray("FulaClient", "put in fulaClient returned put_cid=", put_cid)
            return put_cid
        }
        override fun get(cid: ByteArray): ByteArray? {
            try {
                logByteArray("FulaClient", "get in fulaClient cid=", cid)
                val get_data = fulaClient.get(cid)
                logByteArray("FulaClient", "get in fulaClient returned get_data=", get_data)
                return get_data
            } catch (e: Exception) {
                Log.e("AppMock", "An error occurred while getting data", e)
                return null
            }
        }
        
    }
    @get:Rule
    val mainActivityRule = ActivityScenarioRule(MainActivity::class.java)
    @Test
    fun wnfs_overall() {
        val useInMemoryStore = false  // Or determine this from some configurations or conditions
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
        val client: land.fx.wnfslib.Datastore = if (useInMemoryStore) {
            InMemoryDatastore()
        } else {
            ConvertFulaClient(fulaClient)
        }

        Log.d("AppMock", "client created with id="+fulaClient.id())

        val keyPhrase = ("test").toByteArray(StandardCharsets.UTF_8)
        val digest: MessageDigest = MessageDigest.getInstance("SHA-256");
        val wnfsKey: ByteArray = digest.digest(keyPhrase);
        
        /*
        val sampleData = arrayOf<Int>(98, 59, -76, 96, 127, -7, 64, 73, 54, -102, 67, -108, -31, 23, -34, -43, -50, -104, -32, 71, 56, -39, 116, -28, 80, 27, -85, 93, 86, 5, -87, 119, -86, -78, 124, 52, 104, 72, 11, 8, 40, -22, -106, -1, 20, -109, -6, -70, -5, 3, 25, 89, 35, -23, -77, 73, 116, -1, -42, 39, -110, -49, -77, 71, 35, 71, 117, -19, -34, -43, -26, 90, 102, -68, 105, -13, 65, -121, -108, 18, -109, 99, -90, -111, 98, 81, 66, 19, 82, -18, 60, 0, 75, -120, 126, 76, -87, 105, -123, 0, 97, 9, 96, 86, -84, -62, 113, 92, -26, 121, -48, 40, 90, -97, -126, -114, -87, 116, 56, 29, -11, -79, -80, -62, 91, 112, -105, 38, 51, -81, -91, -95, 33, 109, -55, 41, -126, 87, -57, 90, -13, 109, 102, 78, -26, -94, 113, 62, 40, 65, -30, -109, 48, -85, 123, -21, 120, -10, -39, 62, -75, 108, -22, -111, 30, 115, 21, 57, -29, 23, -3, -39, -21, 70, 36, -77, -98, 33, -2, -4, 82, 46, 14, 85, -26, 108, -77, -128, -45, -79, -83, 110, -80, -77, 56, 66, -25, 83, -66, 64, -78, 27, -38, 124, -21, -32, -124, 90, -53, 36, -59, 36, 61, 116, -59, 96, 14, -109, -75, -97, -41, 111, 114, -111, -90, -1, 31, -74, 121, -96, 118, -52, -111, -122, -18, -51, 12, 110, -25, 103, -95, 120, 40, 63, 108, 117, 78, -108, 69, 72, 104, -90, -82, 14, 34, 120, 98, 11, 37, -42, 50, -123, -1, 53, 98, -111, 30, 78, 82, 68, -83, 124, -11, 47, -120, -107, 61, -22, -23, -117, -77, -38, -109, 88, -89, -124, -13, 68, 1, 125, -102, -20, 7, -119, 34, -73, -43, 18, -106, 108, -20, 106, -103, 11, 34, 52, -37, 49, -107, -36, -81, 90, 36, 102, 108, 2, 83, 69, -28, 123, 96, 85, 109, -108, 0, 1, 106, 99, 74, -26, -80, 65, -93, -30, 71, -93, 41, -110, -12, 41, -97, -57, -32, 52, -107, 23, -125, 71, -47, 110, -126, 54, 50, 43, 55, 12, -16, -18, -106, -82, -51, -88, 19, 89, 61, 74, -72, 4, -100, 57, -48, -106, -14, 26, -15, 63, -65, -22, 124, -77, -93, -20, -14, 117, 59, -96, 10, -43, 10, -83, 100, -15, -7, 11, 26, -63, -68, -35, -104, 83, 98, 113, 20, 62, 61, 63, 81, 93, 47, -30, -56, -53, -17, 126, 44, 44, -85, -13, 87, 88, 115, -85, 88, 11, -47, 67, -53, -63, -2, -19, -35, 126, -31, -72, -118, 71, 90, 99, -80, 53, -108, 47, -26, -122, 6, -74, 86, 27, 122, 85, 53, -34, -1, 107, -94, 13, -42, -7, -84, -23, 73, 2, -86, -76, -58, -88, 118, 114, -114, 2, 122, 117, 29, -111, 64, -43, 123, 115, 110, 88, -37, -54, -119, -123, -119, -47, -119, 53, -7, 9, 65, -117, -117, 20, -124, 48, -36, 71, 18, 82, 98, -117, 120, 11, 39, 30, -76, 34, -30, -66, 111, -27, -125, -37, 8, 81, -38, -11, -112, -14, 65, -7, 44, 107, -53, 114, -49, -127, -124, 1, -122, 1, 113, -39, 102, -36, 117, 104, -120, 27, 87, -94, -81, 123, -126, -43, 61, -50, 25, -91, -63, -4, -121, -127, -20, 83, -55, -35, 67, -68, -77, -93, 72, -44, -71, 15, 59, 63, 107, -21, 7, -96, -82, -47, -120, -120, -69, 116, 97, -97, 12, 44, 41, -31, -66, 119, 35, -97, 116, 17, -107, 67, -14, 106, -110, -94, -62, 89, 29, -86)
        val b = ByteArray(sampleData.size)
        for((index, element) in sampleData.withIndex()) {
            b[index] = element.toByte()
        }
        val codec = (85).toLong()
        Log.d("AppMock", "sampleData is created")
        val testPutCid = client.put(b,codec)
        Log.d("AppMock", "put test was successful=$testPutCid")
        val testData = client.get(testPutCid)
        Log.d("AppMock", "get test was successful=$testData")
        */

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
            Log.d("AppMock", "ls_initial. fileNames_initial="+String(fileNames_initial))
        }  catch (e: Exception) {
            val contains = e.message?.contains("find", true)
            Log.d("AppMock", "ls_initial. error="+e.message)
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


        //create 15 more files
        for (i in 0 until 15) {
            // Create test file for write stream with the current iterator value added to the content and file name
            val testContent = "Hello, World${i + 2}!".toByteArray()
            
            val file = File(pathString, "test${i + 2}.txt")
            // Create a new file
            val isNewFileCreated = file.createNewFile()
            
            if (isNewFileCreated) {
                Log.d("AppMock", "$pathString/test${i + 2}.txt is created successfully.")
            } else {
                Log.d("AppMock", "$pathString/test${i + 2}.txt already exists.")
            }
            //assertTrue(isNewFileCreated)
            file.writeBytes(testContent)
        }
        

/* 
        try {
            val config_err = writeFileFromPath(client, config.cid, "root/testfrompath.txt", "file://"+pathString+"/test.txt")
            Log.d("AppMock", "config_err writeFile. config_err="+config_err)
        } catch (e: Exception) {
            assertNotNull("config should not be null", e)
            Log.d("AppMock", "config_err Error catched "+e.message);
        }
 */       
        config = writeFileFromPath(client, config.cid, "root/testfrompath.txt", pathString+"/test.txt") //target folder does not need to exist
        Log.d("AppMock", "config writeFileFromPath. cid="+config.cid)
        assertNotNull("config should not be null", config)
        assertNotNull("cid should not be null", config.cid)
        
        val fileNames_initial: ByteArray = ls(
            client 
            , config.cid 
            , "root"
        )
        Log.d("AppMock", "ls_initial. fileNames_initial="+String(fileNames_initial))
        // assertNull(String(fileNames_initial))

        val contentfrompath = readFile(client, config.cid, "root/testfrompath.txt")
        assert(contentfrompath contentEquals "Hello, World!".toByteArray())
        Log.d("AppMock", "readFile. content="+String(contentfrompath))

        for (i in 0 until 15) {
            Log.d("AppMock", "writing file ${i + 2}")
            config = writeFileStreamFromPath(client, config.cid, "root/testfrompathstream${i + 2}.txt", "$pathString/test${i + 2}.txt") //target folder does not need to exist
            Log.d("AppMock", "config writeFileStreamFromPath${i + 2}. cid=${config.cid}")
            assertNotNull("config should not be null", config)
            assertNotNull("cid should not be null", config.cid)
        
            Log.d("AppMock", "reading file ${i + 2}")
            val contentstreamfrompathtopath: String = readFilestreamToPath(client, config.cid, "root/testfrompathstream${i + 2}.txt", "$pathString/teststream${i + 2}.txt")
            Log.d("AppMock", "contentstreamfrompathtopath${i + 2}=$contentstreamfrompathtopath")
            assertNotNull("contentstreamfrompathtopath${i + 2} should not be null", contentstreamfrompathtopath)
            val readcontentstream: ByteArray = File(contentstreamfrompathtopath).readBytes()
            assert(readcontentstream contentEquals "Hello, World${i + 2}!".toByteArray())
            Log.d("AppMock", "readFileFromPathOfReadstreamTo. content=${String(readcontentstream)}")
        }

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

        config = cp(client, config.cid, "root/testfrompath.txt", "opt/testfrompathcp.txt") //target folder must exists
        val content_cp = readFile(client, config.cid, "opt/testfrompathcp.txt")
        Log.d("AppMock", "cp. content_cp="+String(content_cp))
        assert(content_cp contentEquals "Hello, World!".toByteArray())

        config = mv(client, config.cid, "root/testfrompath.txt", "root/testfrompathmv.txt") //target folder must exists
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

        config = rm(client, config.cid, "opt/testfrompathcp.txt")
       try {
            readFile(client, config.cid, "opt/testfrompathcp.txt")
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
        assert(content contentEquals "Hello, World!".toByteArray())
        Log.d("AppMock", "readFile. content="+String(content))

        Log.d("AppMock", "****************** Teting large file write and read *******************")
        Log.d("AppMock", "config passed to largefile. cid="+config.cid)
        val file_large = generateLargeTestFile(pathString)
        Log.d("AppMock", "Large file created");
        config = writeFileStreamFromPath(client, config.cid, "root/largeTestFile.txt", pathString+"/largeTestFile.txt") //target folder does not need to exist
        Log.d("AppMock", "config writeFileStreamFromPath for large file. cid="+config.cid)
        assertNotNull("config should not be null for large file", config)
        assertNotNull("cid should not be null for large file", config.cid)

        val largefilecontentstreamfrompathtopath: String = readFilestreamToPath(client, config.cid, "root/largeTestFile.txt", pathString+"/largeTestFileReadStream.txt")
        assertNotNull("contentstreamfrompathtopath for large file should not be null", largefilecontentstreamfrompathtopath)
        val largefile = File(largefilecontentstreamfrompathtopath)

        val fileSizeInBytes = largefile.length()
        val originalfileSizeInBytes = file_large.length()
        assertEquals(fileSizeInBytes, originalfileSizeInBytes)

        Log.d("AppMock", "*************** All tests before reload passed *********************")

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

        val contentfrompathtopath_reloaded: String = readFileToPath(client, config.cid, "root/test.txt", pathString+"/testreload.txt")
        Log.d("AppMock", "contentfrompathtopath_reloaded="+contentfrompathtopath_reloaded)
        assertNotNull("contentfrompathtopath_reloaded should not be null", contentfrompathtopath_reloaded)
        val readcontent_reloaded: ByteArray = File(contentfrompathtopath_reloaded).readBytes()
        Log.d("AppMock", "readFileFromPathOfReadTo. content="+String(readcontent_reloaded))
        assert(readcontent_reloaded contentEquals "Hello, World!".toByteArray())

        Log.d("AppMock", "All tests after reload is passed.")


        // Cleanup phase to delete all generated files
        for (i in 0 until 15) {
            val fileToDelete = File("$pathString/test${i + 2}.txt")
            if (fileToDelete.exists()) {
                val deletionSuccess = fileToDelete.delete()
                if (deletionSuccess) {
                    Log.d("AppMock", "File test${i + 2}.txt deleted successfully.")
                } else {
                    Log.d("AppMock", "Failed to delete file test${i + 2}.txt.")
                }
            }

            val streamFileToDelete = File("$pathString/teststream${i + 2}.txt")
            if (streamFileToDelete.exists()) {
                val deletionSuccess = streamFileToDelete.delete()
                if (deletionSuccess) {
                    Log.d("AppMock", "File teststream${i + 2}.txt deleted successfully.")
                } else {
                    Log.d("AppMock", "Failed to delete file teststream${i + 2}.txt.")
                }
            }
        }
        Log.d("AppMock", "Clean up done.")

    }
}

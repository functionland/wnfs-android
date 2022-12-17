package land.fx.app

import android.util.Log
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.rule.ActivityTestRule
import fulamobile.Fulamobile
import land.fx.wnfslib.Fs.*
import land.fx.wnfslib.Config
import org.junit.Assert.*
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File
import java.nio.charset.StandardCharsets
import java.security.MessageDigest


@RunWith(AndroidJUnit4::class)
class WNFSTest {
    class ConvertFulaClient(private val fulaClient: fulamobile.Client): land.fx.wnfslib.Datastore{
        override fun put(data: ByteArray, codec: Long): ByteArray{
            return fulaClient.put(data, codec)
        }
        override fun get(cid: ByteArray): ByteArray{

            return fulaClient.get(cid)
        }
    }
    @get:Rule
    val mainActivityRule = ActivityTestRule(MainActivity::class.java)
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
        
        /* 
        val sampleData = arrayOf<Int>(36, 212, 208, 119, 240, 231, 139, 103, 180, 43, 238, 192, 90, 25, 53, 111, 134, 254, 216, 237, 50, 10, 16, 85, 157, 154, 111, 204, 182, 200, 71, 44, 39, 18, 115, 1, 24, 175, 92, 3, 132, 226, 156, 7, 131, 220, 159, 77, 240, 190, 224, 11, 55, 203, 198, 194, 76, 39, 150, 181, 23, 232, 32, 23, 249, 162, 213, 93, 225, 191, 100, 168, 161, 234, 48, 232, 219, 38, 31, 230, 114, 53, 35, 10, 222, 212, 16, 92, 83, 157, 235, 202, 189, 48, 192, 165, 58, 70, 32, 142, 227, 151, 23, 175, 124, 41, 127, 145, 31, 167, 182, 205, 132, 151, 129, 58, 65, 136, 26, 32, 35, 221, 75, 76, 165, 128, 82, 219, 155, 216, 167, 219, 253, 155, 46, 130, 88, 44, 7, 143, 32, 111, 191, 238, 87, 40, 146, 46, 247, 135, 181, 29, 13, 38, 183, 251, 99, 190, 100, 234, 182, 121, 109, 253, 45, 54, 250, 94, 251, 158, 158, 144, 198, 253, 37, 111, 22, 87, 6, 166, 119, 123, 89, 230, 64, 90, 244, 205, 249, 168, 110, 51, 118, 60, 226, 102, 120, 129, 65, 3, 170, 172, 45, 119, 120, 62, 111, 55, 21, 91, 86, 53, 184, 253, 59, 41, 0, 90, 145, 41, 230, 109, 108, 46, 118, 225, 176, 221, 98, 221, 53, 92, 119, 254, 57, 163, 120, 179, 160, 74, 203, 104, 35, 188, 96, 153, 178, 122, 214, 30, 254, 19, 6, 33, 183, 14, 223, 7, 161, 171, 245, 21, 214, 1, 60, 79, 54, 202, 67, 145, 12, 240, 60, 210, 115, 217, 171, 175, 135, 33, 24, 138, 181, 63, 44, 4, 105, 76, 54, 230, 217, 81, 141, 209, 196, 129, 236, 18, 155, 246, 6, 138, 239, 8, 170, 10, 50, 233, 104, 50, 233, 58, 140, 16, 16, 213, 238, 95, 239, 227, 2, 108, 0, 215, 14, 84, 86, 82, 241, 50, 174, 1, 72, 132, 150, 3, 56, 177, 156, 254, 189, 7, 177, 2, 207, 107, 243, 234, 64, 42, 74, 245, 230, 240, 116, 193, 222, 29, 187, 239, 240, 185, 49, 27, 132, 169, 121, 86, 186, 10, 226, 150, 237, 154, 75, 173, 198, 20, 213, 215, 11, 29, 251, 140, 81, 249, 225, 148, 169, 197, 201, 180, 206, 252, 219, 130, 224, 152, 60, 252, 121, 236, 214, 107, 15, 172, 104, 17, 64, 99, 57, 253, 251, 114, 64, 196, 184, 36, 181, 208, 181, 80, 19, 143, 75, 250, 0, 55, 211, 135, 59, 126, 151, 11, 149, 59, 8, 248, 156, 45, 244, 62, 26, 53, 136, 197, 203, 99, 122, 225, 67, 116, 165, 254, 172, 29, 170, 116, 59, 214, 47, 213, 45, 60, 226, 209, 229, 98, 4, 37, 33, 156, 140, 243, 176, 23, 31, 149, 102, 89, 118, 126, 6, 66, 185, 188, 187, 241, 183, 187, 234, 132, 137, 28, 30, 1, 104, 151, 64, 238, 220, 139, 144, 22, 87, 215, 13, 90, 156, 79, 71, 39, 41, 123, 114, 128, 5, 223, 97, 14, 47, 3, 178, 15, 102, 52, 98, 127, 226, 163, 70, 237, 241, 83, 90, 172, 178, 66, 241, 123, 213, 104, 242, 47, 47, 2, 198, 194, 150, 123, 181, 86, 50, 86, 190, 48, 15, 60, 188, 77, 135, 1, 118, 134, 47, 230, 103, 16, 230, 120, 255, 85, 86, 101, 236, 35, 127, 147, 234, 226, 93, 174, 200, 185, 45, 36, 117, 13, 253, 151, 232, 80, 106, 5, 46, 167, 123, 234, 218, 182, 66, 183, 0, 120, 107, 251, 116, 80, 79, 212, 142, 160, 8, 217, 149, 161, 238, 219, 141, 127, 87, 180, 192, 40, 154, 31, 164, 184, 117, 158, 167, 210, 119, 219, 238, 166, 234, 84, 239, 128, 143, 140, 171, 171, 181, 108, 96, 215, 125, 161, 92, 179, 48, 222, 115, 25, 29, 27, 68, 240, 139, 149, 10, 215, 14, 185, 219, 10, 236, 112, 128, 68, 226, 17, 45, 100, 135, 232, 222, 36, 193, 233, 4, 51, 222, 61, 103, 108, 229, 177, 177, 205, 86, 82, 95, 185, 248, 115, 1, 215, 197, 123, 9, 155, 152, 37, 233, 212, 150, 163, 33, 188, 4, 99, 84, 45, 5, 199, 65, 188, 226, 87, 165, 35, 108, 180, 135, 159, 55, 242, 22, 21, 234, 194, 225, 246, 193, 218, 246, 2, 224, 2, 103, 50, 121, 105, 113, 159, 5, 238, 40, 223, 184, 61, 79, 215, 118, 216, 154, 112, 39, 27, 56, 233, 80, 218, 40, 204, 213, 14, 171, 220, 12, 62, 142, 213, 118, 23, 102, 122, 148, 194, 168, 62, 183, 250, 232, 134, 84, 49, 149, 102, 141, 144, 42, 134, 211, 44, 185, 136, 186, 172, 146, 215, 197, 76, 125, 172, 114, 115, 126, 42, 66, 1, 224, 222, 59, 49, 58, 22, 49, 132, 27, 1, 79, 158, 82, 20, 57, 120, 130, 114, 55, 141, 237, 39, 249, 6, 161, 98, 14, 151, 149, 243, 150, 179, 238, 232, 207, 214, 98, 179, 114, 22, 130, 72, 229, 17, 180, 156, 211, 211, 190, 244, 10, 20, 182, 30, 87, 18, 164, 101, 102, 28, 31, 236, 142, 239, 183, 25, 234, 166, 8, 230, 42, 94, 239, 73, 138, 186, 28, 88, 111, 233, 105, 66, 176, 88, 245, 249, 156, 41, 226, 44, 24, 114, 244, 193, 25, 182, 38, 155, 28, 24, 237, 62, 251, 254, 153, 2, 31, 244, 206, 230, 83, 122, 13, 77, 140, 27, 200, 247, 19, 24, 241, 169, 44, 252, 64, 106, 139, 131, 208, 42, 129, 249, 179, 134, 244, 225, 107, 219, 68, 126, 105, 179, 242, 140, 198, 77, 2, 255, 163, 222, 190, 2, 198, 200, 130, 245, 125, 238, 226, 149, 130, 106, 109, 176, 145, 18, 233, 156, 148, 174, 251, 224, 181, 68, 34, 94, 243, 1, 150, 98, 96, 63, 157, 128, 49, 230, 250, 97, 131, 48, 74, 125, 228, 88, 4, 213, 127, 201, 60, 17, 41, 246, 140, 35, 11, 172, 36, 118, 238, 162, 17, 22, 189, 188, 102, 83, 67, 242, 215, 78, 231, 232, 246, 202, 52, 49, 87, 119, 28, 14, 225, 28, 86, 45, 73, 190, 128, 196, 255, 106, 196, 63, 107, 122, 7, 14, 254, 7, 255, 245, 71, 193, 241, 138, 93, 74, 177, 165, 25, 8, 67, 3, 45, 113, 182, 203, 89, 179, 173, 156, 156, 248, 64, 233, 194, 67, 214, 62, 211, 236, 90, 183, 203, 183, 2, 95, 172, 84, 143, 26, 223, 48, 208, 151, 152, 110, 132, 88, 246, 62, 118, 95, 116, 116, 6, 228, 92, 6, 199, 70, 202, 147, 160, 177, 33, 30, 227, 107, 78, 45, 44, 99, 172, 69, 182, 189, 173, 160, 25, 98, 62, 177, 73, 83, 140, 196, 9, 174, 197, 71, 235, 128, 60, 148, 175, 30, 162, 180, 44, 165, 247, 65, 87, 169, 71, 178, 237, 25, 72, 191, 239, 222, 128, 184, 32, 214, 160, 95, 96, 85, 58, 110, 118, 179, 133, 110, 134, 29, 111, 199, 220, 90, 12, 83, 9, 12, 106, 124, 51, 149, 172, 146, 145, 32, 244, 15, 53, 218, 6, 117, 38, 4, 186, 43, 235, 202, 162, 134, 62, 44, 14, 31, 8, 49, 101, 169, 32, 101, 242, 58, 38, 34, 250, 65, 211, 46, 235, 12, 143, 76, 245, 89, 69, 193, 210, 99, 30, 57, 43, 227, 154, 248, 108, 187, 152, 7, 91, 122, 148, 211, 182, 147, 162, 117, 66, 126, 108, 144, 229, 101, 224, 188, 45, 174, 48, 21, 224, 10, 0, 36, 98, 243, 214, 186, 101, 71, 53, 46, 106, 95, 157, 179, 77, 192, 20, 74, 159, 223, 158, 185, 23)
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

        val privateForest = createPrivateForest(client)
        Log.d("AppMock", "privateForest created=$privateForest")
        println("&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&")
        println(privateForest)


        var config = createRootDir(client, privateForest, wnfsKey)
        Log.d("AppMock", "config crecreateRootDirated. cid="+config.cid+" & private_ref="+config.private_ref)
        assertNotNull("cid should not be null", config.cid)
        assertNotNull("private_ref should not be null", config.private_ref)

        val fileNames_initial: String = ls(
            client 
            , "bafyreieqp253whdfdrky7hxpqezfwbkjhjdbxcq4mcbp6bqf4jdbncbx4y" 
            , "{\"saturated_name_hash\":[229,31,96,28,24,238,207,22,36,150,191,37,235,68,191,144,219,250,5,97,85,208,156,134,137,74,25,209,6,66,250,127],\"content_key\":[172,199,245,151,207,21,26,76,52,109,93,57,118,232,9,230,149,46,37,137,174,42,119,29,102,175,25,149,213,204,45,15],\"revision_key\":[17,5,78,59,8,135,144,240,41,248,135,168,222,186,158,240,100,10,129,4,180,55,126,115,146,239,22,177,207,118,169,51]}"
            , "root/"
        )
        Log.d("AppMock", "ls_initial. fileNames_initial="+fileNames_initial)

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

/* 
        try {
            val config_err = writeFileFromPath(client, config.cid, config.private_ref, "root/testfrompath.txt", "file://"+pathString+"/test.txt")
            Log.d("AppMock", "config_err writeFile. config_err="+config_err)
        } catch (e: Exception) {
            assertNotNull("config should not be null", e)
            Log.d("AppMock", "config_err Error catched "+e.message);
        }
 */       

        config = writeFileFromPath(client, config.cid, config.private_ref, "root/testfrompath.txt", pathString+"/test.txt")
        Log.d("AppMock", "config writeFile. cid="+config.cid+" & private_ref="+config.private_ref)
        assertNotNull("config should not be null", config)
        assertNotNull("cid should not be null", config.cid)
        


        val contentfrompath = readFile(client, config.cid, config.private_ref, "root/testfrompath.txt")
        assert(contentfrompath contentEquals "Hello, World!".toByteArray())
        Log.d("AppMock", "readFileFromPath. content="+contentfrompath.toString())


        val contentfrompathtopath: String = readFileToPath(client, config.cid, config.private_ref, "root/testfrompath.txt", pathString+"/test2.txt")
        Log.d("AppMock", "contentfrompathtopath="+contentfrompathtopath)
        assertNotNull("contentfrompathtopath should not be null", contentfrompathtopath)
        val readcontent: ByteArray = File(contentfrompathtopath).readBytes()
        assert(readcontent contentEquals "Hello, World!".toByteArray())
        Log.d("AppMock", "readFileFromPathOfReadTo. content="+String(readcontent))

        config = rm(client, config.cid, config.private_ref, "root/testfrompath.txt")
        val content2 = readFile(client, config.cid, config.private_ref, "root/testfrompath.txt")
        Log.d("AppMock", "rm. content="+String(content2))
        assert(content2 contentEquals "".toByteArray())


        config = writeFile(client, config.cid, config.private_ref, "root/test.txt", "Hello, World!".toByteArray())
        assertNotNull("cid should not be null", config.cid)
        Log.d("AppMock", "config writeFile. cid="+config.cid+" & private_ref="+config.private_ref)

        config = mkdir(client,  config.cid, config.private_ref, "root/test1")
        Log.d("AppMock", "config mkdir. cid="+config.cid+" & private_ref="+config.private_ref)

        val fileNames: String = ls(client, config.cid, config.private_ref, "root")
        Log.d("AppMock", "ls. fileNames="+fileNames)
        //assertEquals(fileNames, "[{\"name\":\"test.txt\",\"creation\":\"2022-12-17 00:36:02 UTC\",\"modification\":\"2022-12-17 00:36:02 UTC\"},{\"name\":\"test1\",\"creation\":\"\",\"modification\":\"]\"}]")
        

        val content = readFile(client, config.cid, config.private_ref, "root/test.txt")
        assert(content contentEquals "Hello, World!".toByteArray())
        Log.d("AppMock", "readFile. content="+content.toString())

        Log.d("AppMock", "All tests before reload passed")

        Log.d("AppMock", "wnfs12 Testing reload with cid="+config.cid+" & wnfsKey="+wnfsKey.toString())
        //Testing reload Directory
        var private_ref_reload: String = getPrivateRef(client, wnfsKey, config.cid)
        Log.d("AppMock", "wnfs12 original PrivateRef. private_ref="+config.private_ref)
        Log.d("AppMock", "wnfs12 getPrivateRef. private_ref="+private_ref_reload)
        assertNotNull("private_ref should not be null", private_ref_reload)

        val fileNames_reloaded: String = ls(client, config.cid, private_ref_reload, "root")
        //assertEquals(fileNames_reloaded, "test.txt\ntest1")
        

        val content_reloaded = readFile(client, config.cid, private_ref_reload, "root/test.txt")
        Log.d("AppMock", "readFile. content="+content_reloaded.toString())
        assert(content_reloaded contentEquals "Hello, World!".toByteArray())

        val contentfrompathtopath_reloaded: String = readFileToPath(client, config.cid, private_ref_reload, "root/test.txt", pathString+"/test2.txt")
        Log.d("AppMock", "contentfrompathtopath_reloaded="+contentfrompathtopath_reloaded)
        assertNotNull("contentfrompathtopath_reloaded should not be null", contentfrompathtopath_reloaded)
        val readcontent_reloaded: ByteArray = File(contentfrompathtopath_reloaded).readBytes()
        assert(readcontent_reloaded contentEquals "Hello, World!".toByteArray())
        Log.d("AppMock", "readFileFromPathOfReadTo. content="+readcontent_reloaded.toString())

    }
}

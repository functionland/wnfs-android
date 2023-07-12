package land.fx.wnfslib;

import android.util.Log;

import androidx.annotation.NonNull;

import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.LinkedList;
import java.util.List;
import org.json.JSONObject;
import org.json.JSONArray;
import land.fx.wnfslib.*;;

public final class Fs {

    private static native String initNative(Datastore datastore, byte[] wnfsKey);

    private static native String loadWithWNFSKeyNative(Datastore datastore, byte[] wnfsKey, String cid);

    private static native Config writeFileFromPathNative(Datastore datastore, String cid, String path, String filename);

    private static native Config writeFileNative(Datastore datastore, String cid, String path, byte[] content);

    private static native byte[] lsNative(Datastore datastore, String cid, String path);

    private static native Config mkdirNative(Datastore datastore, String cid, String path);

    private static native Config rmNative(Datastore datastore, String cid, String path);

    private static native Config mvNative(Datastore datastore, String cid, String sourcePath, String targetPath);

    private static native Config cpNative(Datastore datastore, String cid, String sourcePath, String targetPath);

    private static native String readFileToPathNative(Datastore datastore, String cid, String path, String filename);

    private static native String readFilestreamToPathNative(Datastore datastore, String cid, String path, String filename);
    
    private static native byte[] readFileNative(Datastore datastore, String cid, String path);

    class WnfsException extends Exception
{

      public WnfsException() {}

      // Constructor that accepts a message
      public WnfsException(String func ,String reason)
      {
         super(String.format("An Error Occured in Fs.%s: %s", func, reason));
      }
 }

    @NonNull
    public static Config init(Datastore datastore, byte[] wnfsKey) throws Exception {
        try {
            ConfigResult res = initNative(datastore, cid, path, filename);
            if(res != null && res.ok()) {
                return res.getResult();
            } else {
                throw new WnfsException("Fs.init", res.getReason());
            }
        }
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    @NonNull
    public static void loadWithWNFSKey(Datastore datastore, byte[] wnfsKey, String cid) throws Exception {
        try {
            Result res = loadWithWNFSKeyNative(datastore, wnfsKey, cid);
            if(res == null || !res.ok()) {
                throw new WnfsException("Fs.loadWithWNFSKey", res.getReason());
            }
        }
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    @NonNull
    public static Config writeFileFromPath(Datastore datastore, String cid, String path, String filename) throws Exception {
        try {
            ConfigResult res = writeFileFromPathNative(datastore, cid, path, filename);
            if(res != null && res.ok()) {
                return res.getResult();
            } else {
                throw new WnfsException("Fs.writeFileFromPath", res.getReason());
            }
        } 
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    @NonNull
    public static Config writeFile(Datastore datastore, String cid, String path, byte[] content) throws Exception {
        try {
            ConfigResult res = writeFileNative(datastore, cid, path, content);
            if(res != null && res.ok()) {
                return res.getResult();
            } else {
                throw new WnfsException("Fs.writeFile", res.getReason());
            }
        } 
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    @NonNull
    public static byte[] ls(Datastore datastore, String cid, String path) throws Exception {
        try {
            
            Log.d("wnfs", "JSONArray is reached2");
            BytesResult res = lsNative(datastore, cid, path, content);
            Log.d("wnfs", "lsResult is reached: ");
            if(res != null && res.ok()) {
                return res.getResult();
            } else {
                throw new WnfsException("Fs.writeFile", res.getReason());
            }
            /*JSONArray output = new JSONArray();
            byte[] rowSeparatorPattern = {33, 33, 33}; //!!!
            byte[] itemSeparatorPattern = {63, 63, 63}; //???
            List<byte[]> rows = split(rowSeparatorPattern, lsResult);
            for (byte[] element : rows) {
                JSONObject obj = new JSONObject();
                List<byte[]> rowDetails = split(itemSeparatorPattern, element);
                if (!rowDetails.isEmpty()) {
                    String name = new String(rowDetails.get(0), StandardCharsets.UTF_8);
                    if(!name.isEmpty()) {
                        obj.put("name", name);
                        if(rowDetails.size() >= 2) {
                            String creation = new String(rowDetails.get(1), StandardCharsets.UTF_8);
                            obj.put("creation", creation);
                        } else {
                            obj.put("creation", "");
                        }
                        if(rowDetails.size() >= 3) {
                            String modification = new String(rowDetails.get(2), StandardCharsets.UTF_8);
                            obj.put("modification", modification);
                        } else {
                            obj.put("modification", "");
                        }
                        output.put(obj);
                    }
                }
                
            }*/
        }
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    @NonNull
    public static Config mkdir(Datastore datastore, String cid, String path) throws Exception {
        try {
            ConfigResult res = mkdirNative(datastore, cid, path);
            if(res != null && res.ok()) {
                return res.getResult();
            } else {
                throw new WnfsException("Fs.mkdir", res.getReason());
            }
        } 
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    @NonNull
    public static Config rm(Datastore datastore, String cid, String path) {
        try {
            ConfigResult res = rmNative(datastore, cid, path);
            if(res != null && res.ok()) {
                return res.getResult();
            } else {
                throw new WnfsException("Fs.rm", res.getReason());
            }
        } 
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    @NonNull
    public static Config mv(Datastore datastore, String cid, String sourcePath, String targetPath) {
        try {
            ConfigResult res = mvNative(datastore, cid, sourcePath, targetPath);
            if(res != null && res.ok()) {
                return res.getResult();
            } else {
                throw new WnfsException("Fs.mv", res.getReason());
            }
        } 
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    @NonNull
    public static Config cp(Datastore datastore, String cid, String sourcePath, String targetPath) {
        try {
            ConfigResult res = cpNative(datastore, cid, sourcePath, targetPath);
            if(res != null && res.ok()) {
                return res.getResult();
            } else {
                throw new WnfsException("Fs.cp", res.getReason());
            }
        } 
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    @NonNull
    public static String readFileToPath(Datastore datastore, String cid, String path, String filename) throws Exception {
        try{
            StringResult res = readFileToPathNative(datastore, cid, path, filename);
            if(res != null && !res.isEmpty()) {
                return res.getResult();
            } else {
                throw new WnfsException("Fs.readFileToPathNative", res.getReason());
            }
        }
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    @NonNull
    public static String readFilestreamToPath(Datastore datastore, String cid, String path, String filename) throws Exception {
        try{
            StringResult res = readFilestreamToPathNative(datastore, cid, path, filename);
            if(res != null && !res.isEmpty()) {
                return res.getResult();
            } else {
                throw new WnfsException("Fs.readFilestreamToPathNative", res.getReason());
            }
        }
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    public static byte[] readFile(Datastore datastore, String cid, String path) {
        try{
            BytesResult res = readFileNative(datastore, cid, path);
            if(res != null && !res.isEmpty()) {
                return res.getResult();
            } else {
                throw new WnfsException("Fs.readFileNative", res.getReason());
            }
        }
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    public static native void initRustLogger();

    private static boolean isMatch(@NonNull byte[] pattern, byte[] input, int pos) {
        for(int i=0; i< pattern.length; i++) {
            if(pattern[i] != input[pos+i]) {
                return false;
            }
        }
        return true;
    }

    @NonNull
    private static List<byte[]> split(byte[] pattern, @NonNull byte[] input) {
        List<byte[]> l = new LinkedList<>();
        int blockStart = 0;
        for(int i=0; i<input.length; i++) {
            if(isMatch(pattern,input,i)) {
                l.add(Arrays.copyOfRange(input, blockStart, i));
                blockStart = i+pattern.length;
                i = blockStart;
            }
        }
        l.add(Arrays.copyOfRange(input, blockStart, input.length ));
        return l;
    }
}



package land.fx.wnfslib;

import android.util.Log;

import androidx.annotation.NonNull;

import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.LinkedList;
import java.util.List;
import org.json.JSONObject;
import org.json.JSONArray;

public final class Fs {

    private static native String createPrivateForestNative(Datastore datastore);

    private static native String getPrivateRefNative(Datastore datastore, byte[] wnfsKey, String cid);

    private static native Config createRootDirNative(Datastore datastore, String cid, byte[] wnfsKey);

    private static native Config writeFileFromPathNative(Datastore datastore, String cid, String privateRef, String path, String filename);

    private static native Config writeFileNative(Datastore datastore, String cid, String privateRef, String path, byte[] content);

    private static native byte[] lsNative(Datastore datastore, String cid, String privateRef, String path);

    private static native Config mkdirNative(Datastore datastore, String cid, String privateRef, String path);

    private static native Config rmNative(Datastore datastore, String cid, String privateRef, String path);

    private static native String readFileToPathNative(Datastore datastore, String cid, String privateRef, String path, String filename);

    private static native String readFilestreamToPathNative(Datastore datastore, String cid, String privateRef, String path, String filename);
    
    private static native byte[] readFileNative(Datastore datastore, String cid, String privateRef, String path);

    @NonNull
    public static String createPrivateForest(Datastore datastore) throws Exception {
        try {
            String res = createPrivateForestNative(datastore);
            if(res != null && !res.isEmpty()) {
                return res;
            } else {
                throw new Exception("An Error Occured in Fs.createPrivateForest Search logs for wnfsError to find out more.");
            }
        }
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    @NonNull
    public static String getPrivateRef(Datastore datastore, byte[] wnfsKey, String cid) throws Exception {
        try {
            String res = getPrivateRefNative(datastore, wnfsKey, cid);
            if(res != null && !res.isEmpty()) {
                return res;
            } else {
                throw new Exception("An Error Occured in Fs.getPrivateRef Search logs for wnfsError to find out more.");
            }
        }
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    @NonNull
    public static Config createRootDir(Datastore datastore, String cid, byte[] wnfsKey) throws Exception {
        try {
            Config res = createRootDirNative(datastore, cid, wnfsKey);
            if(res != null) {
                return res;
            } else {
                throw new Exception("An Error Occured in Fs.createRootDir. Search logs for wnfsError to find out more.");
            }
        } 
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    @NonNull
    public static Config writeFileFromPath(Datastore datastore, String cid, String privateRef, String path, String filename) throws Exception {
        try {
            Config res = writeFileFromPathNative(datastore, cid, privateRef, path, filename);
            if(res != null) {
                return res;
            } else {
                throw new Exception("An Error Occured in Fs.writeFileFromPath Search logs for wnfsError to find out more.");
            }
        } 
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    @NonNull
    public static Config writeFile(Datastore datastore, String cid, String privateRef, String path, byte[] content) throws Exception {
        try {
            Config res = writeFileNative(datastore, cid, privateRef, path, content);
            if(res != null) {
                return res;
            } else {
                throw new Exception("An Error Occured in Fs.writeFile Search logs for wnfsError to find out more.");
            }
        } 
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    @NonNull
    public static byte[] ls(Datastore datastore, String cid, String privateRef, String path) throws Exception {
        try {
            
            Log.d("wnfs", "JSONArray is reached2");
            byte[] lsResult = lsNative(datastore, cid, privateRef, path);
            Log.d("wnfs", "lsResult is reached: ");
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

            return lsResult;
        }
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    @NonNull
    public static Config mkdir(Datastore datastore, String cid, String privateRef, String path) throws Exception {
        try {
            Config res = mkdirNative(datastore, cid, privateRef, path);
            if(res != null) {
                return res;
            } else {
                throw new Exception("An Error Occured in Fs.mkdir Search logs for wnfsError to find out more.");
            }
        } 
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    public static Config rm(Datastore datastore, String cid, String privateRef, String path) {
        return rmNative(datastore, cid, privateRef, path);
    }

    @NonNull
    public static String readFileToPath(Datastore datastore, String cid, String privateRef, String path, String filename) throws Exception {
        try{
            String res = readFileToPathNative(datastore, cid, privateRef, path, filename);
            if(res != null && !res.isEmpty()) {
                return res;
            } else {
                throw new Exception("An Error Occured in Fs.readFileToPathNative Search logs for wnfsError to find out more.");
            }
        }
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    @NonNull
    public static String readFilestreamToPath(Datastore datastore, String cid, String privateRef, String path, String filename) throws Exception {
        try{
            String res = readFilestreamToPathNative(datastore, cid, privateRef, path, filename);
            if(res != null && !res.isEmpty()) {
                return res;
            } else {
                throw new Exception("An Error Occured in Fs.readFilestreamToPathNative Search logs for wnfsError to find out more.");
            }
        }
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    public static byte[] readFile(Datastore datastore, String cid, String privateRef, String path) {
        return readFileNative(datastore, cid, privateRef, path);
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



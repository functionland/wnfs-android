package land.fx.wnfslib;

import android.util.Log;

import androidx.annotation.NonNull;

import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.LinkedList;
import java.util.List;
import org.json.JSONObject;
import org.json.JSONArray;
import land.fx.wnfslib.result.*;
import land.fx.wnfslib.*;
import land.fx.wnfslib.exceptions.WnfsException;

public final class Fs {

    private static native ConfigResult initNative(Datastore datastore, byte[] wnfsKey);

    private static native Result loadWithWNFSKeyNative(Datastore datastore, byte[] wnfsKey, String cid);

    private static native ConfigResult writeFileFromPathNative(Datastore datastore, String cid, String path, String filename);

    private static native ConfigResult writeFileStreamFromPathNative(Datastore datastore, String cid, String path, String filename);

    private static native ConfigResult writeFileNative(Datastore datastore, String cid, String path, byte[] content);

    private static native BytesResult lsNative(Datastore datastore, String cid, String path);

    private static native ConfigResult mkdirNative(Datastore datastore, String cid, String path);

    private static native ConfigResult rmNative(Datastore datastore, String cid, String path);

    private static native ConfigResult mvNative(Datastore datastore, String cid, String sourcePath, String targetPath);

    private static native ConfigResult cpNative(Datastore datastore, String cid, String sourcePath, String targetPath);

    private static native StringResult readFileToPathNative(Datastore datastore, String cid, String path, String filename);

    private static native StringResult readFilestreamToPathNative(Datastore datastore, String cid, String path, String filename);
    
    private static native BytesResult readFileNative(Datastore datastore, String cid, String path);



    @NonNull
    public static Config init(Datastore datastore, byte[] wnfsKey) throws Exception {
        try {
            ConfigResult res = initNative(datastore, wnfsKey);
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
                throw new WnfsException("Fs.loadWithWNFSKey for cid="+cid, res.getReason());
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
                throw new WnfsException("Fs.writeFileFromPath for cid="+cid, res.getReason());
            }
        } 
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    @NonNull
    public static Config writeFileStreamFromPath(Datastore datastore, String cid, String path, String filename) throws Exception {
        try {
            ConfigResult res = writeFileStreamFromPathNative(datastore, cid, path, filename);
            if(res != null && res.ok()) {
                return res.getResult();
            } else {
                throw new WnfsException("Fs.writeFileStreamFromPath for cid="+cid, res.getReason());
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
                throw new WnfsException("Fs.writeFile for cid="+cid, res.getReason());
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
            BytesResult res = lsNative(datastore, cid, path);
            Log.d("wnfs", "lsResult is reached: ");
            if(res != null && res.ok()) {
                return res.getResult();
            } else {
                throw new WnfsException("Fs.ls for cid="+cid, res.getReason());
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
                throw new WnfsException("Fs.mkdir for cid="+cid, res.getReason());
            }
        } 
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    @NonNull
    public static Config rm(Datastore datastore, String cid, String path) throws Exception {
        try {
            ConfigResult res = rmNative(datastore, cid, path);
            if(res != null && res.ok()) {
                return res.getResult();
            } else {
                throw new WnfsException("Fs.rm for cid="+cid, res.getReason());
            }
        } 
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    @NonNull
    public static Config mv(Datastore datastore, String cid, String sourcePath, String targetPath) throws Exception {
        try {
            ConfigResult res = mvNative(datastore, cid, sourcePath, targetPath);
            if(res != null && res.ok()) {
                return res.getResult();
            } else {
                throw new WnfsException("Fs.mv for cid="+cid, res.getReason());
            }
        } 
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    @NonNull
    public static Config cp(Datastore datastore, String cid, String sourcePath, String targetPath) throws Exception {
        try {
            ConfigResult res = cpNative(datastore, cid, sourcePath, targetPath);
            if(res != null && res.ok()) {
                return res.getResult();
            } else {
                throw new WnfsException("Fs.cp for cid="+cid, res.getReason());
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
            if(res != null && res.ok()) {
                return res.getResult();
            } else {
                throw new WnfsException("Fs.readFileToPathNative for cid="+cid, res.getReason());
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
            if(res != null && res.ok()) {
                return res.getResult();
            } else {
                throw new WnfsException("Fs.readFilestreamToPathNative for cid="+cid, res.getReason());
            }
        }
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    public static byte[] readFile(Datastore datastore, String cid, String path) throws Exception {
        try{
            BytesResult res = readFileNative(datastore, cid, path);
            if(res != null && res.ok()) {
                return res.getResult();
            } else {
                throw new WnfsException("Fs.readFileNative for cid="+cid, res.getReason());
            }
        }
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

    public static native void initRustLogger();

    private static boolean isMatch(@NonNull byte[] pattern, byte[] input, int pos) throws Exception {
        for(int i=0; i< pattern.length; i++) {
            if(pattern[i] != input[pos+i]) {
                return false;
            }
        }
        return true;
    }

    @NonNull
    private static List<byte[]> split(byte[] pattern, @NonNull byte[] input) throws Exception {
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



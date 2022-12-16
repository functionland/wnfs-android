package land.fx.wnfslib;

import androidx.annotation.NonNull;

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

    private static native byte[] readFileNative(Datastore datastore, String cid, String privateRef, String path);

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

    public static byte[] ls(Datastore datastore, String cid, String privateRef, String path) throws Exception {
        try {
            byte[] lsResult = lsNative(datastore, cid, privateRef, path);
            return lsResult;
        }
        catch(Exception e) {
            throw new Exception(e.getMessage());
        }
    }

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

    public static byte[] readFile(Datastore datastore, String cid, String privateRef, String path) {
        return readFileNative(datastore, cid, privateRef, path);
    }

    public static native void initRustLogger();
}



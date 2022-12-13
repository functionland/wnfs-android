package land.fx.wnfslib;

import androidx.annotation.NonNull;

public final class Fs {

    private static native String createPrivateForestNative(Datastore datastore);

    private static native Config createRootDirNative(Datastore datastore, String cid, byte[] wnfsKey, Boolean reload);

    private static native Config writeFileFromPathNative(Datastore datastore, String cid, String privateRef, String path, String filename);

    private static native Config writeFileNative(Datastore datastore, String cid, String privateRef, String path, byte[] content);

    private static native String lsNative(Datastore datastore, String cid, String privateRef, String path);

    private static native Config mkdirNative(Datastore datastore, String cid, String privateRef, String path);

    private static native Config rmNative(Datastore datastore, String cid, String privateRef, String path);

    private static native String readFileToPathNative(Datastore datastore, String cid, String privateRef, String path, String filename);

    private static native byte[] readFileNative(Datastore datastore, String cid, String privateRef, String path);

    public static String createPrivateForest(Datastore datastore) {
        return createPrivateForestNative(datastore);
    }

    public static Config createRootDir(Datastore datastore, String cid, byte[] wnfsKey, Boolean reload) {
        return createRootDirNative(datastore, cid, wnfsKey, reload);
    }

    public static Config writeFileFromPath(Datastore datastore, String cid, String privateRef, String path, String filename) {
        return writeFileFromPathNative(datastore, cid, privateRef, path, filename);
    }

    public static Config writeFile(Datastore datastore, String cid, String privateRef, String path, byte[] content) {
        return writeFileNative(datastore, cid, privateRef, path, content);
    }

    public static String ls(Datastore datastore, String cid, String privateRef, String path) {
        return lsNative(datastore, cid, privateRef, path);
    }

    public static Config mkdir(Datastore datastore, String cid, String privateRef, String path) {
        return mkdirNative(datastore, cid, privateRef, path);
    }

    public static Config rm(Datastore datastore, String cid, String privateRef, String path) {
        return rmNative(datastore, cid, privateRef, path);
    }

    public static String readFileToPath(Datastore datastore, String cid, String privateRef, String path, String filename) {
        return readFileToPathNative(datastore, cid, privateRef, path, filename);
    }

    public static byte[] readFile(Datastore datastore, String cid, String privateRef, String path) {
        return readFileNative(datastore, cid, privateRef, path);
    }

    public static native void initRustLogger();
}


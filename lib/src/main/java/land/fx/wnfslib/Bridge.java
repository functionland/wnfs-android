package land.fx.wnfslib;

import androidx.annotation.NonNull;

public final class Bridge {

    public static final class Config {
        private final String cid;

        private final String private_ref;

        public String getCid() {
            return this.cid;
        }

        public String getPrivate_ref() {
            return this.private_ref;
        }

        public Config(String cid, String private_ref) {
            super();
            this.cid = cid;
            this.private_ref = private_ref;
        }

        @NonNull
        public land.fx.wnfslib.Bridge.Config create(String cid, String private_ref) {
            return new land.fx.wnfslib.Bridge.Config(cid, private_ref);
        }
    }

    public interface Datastore {
        byte[] put(byte[] data, long codec);

        byte[] get(byte[] cid);
    }

    private static native String createPrivateForestNative(Datastore datastore);

    private static native Config createRootDirNative(Datastore datastore, String cid);

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

    public static Config createRootDir(Datastore datastore, String cid) {
        return createRootDirNative(datastore, cid);
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



package land.fx.wnfslib;

public interface Datastore {
    byte[] put(byte[] cid,byte[] data);

    byte[] get(byte[] cid);

    Long getTotalBytesGet();

    Long getTotalBytesPut();
}

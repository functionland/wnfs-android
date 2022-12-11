package land.fx.wnfslib;

public interface Datastore {
    byte[] put(byte[] data, long codec);

    byte[] get(byte[] cid);
}

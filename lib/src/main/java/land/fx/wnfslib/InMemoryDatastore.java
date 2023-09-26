package land.fx.wnfslib;

import land.fx.wnfslib.Config;
import land.fx.wnfslib.Datastore;
import land.fx.wnfslib.result.*;
import land.fx.wnfslib.exceptions.*;
import java.io.File;
import java.util.Base64;
import java.lang.Exception;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import java.util.logging.Logger;


public final class InMemoryDatastore implements Datastore {
    private static Logger LOGGER = Logger.getLogger("InfoLogging");
    private final ConcurrentHashMap<String, byte[]> store = new ConcurrentHashMap<String, byte[]>();
    private final ExecutorService executor = Executors.newSingleThreadExecutor();
    private Long totalBytesPut = 0L; // Keep track of the total bytes written
    private Long totalBytesGet = 0L; // Keep track of the total bytes got

    public InMemoryDatastore() {

    }

    public static String hex(byte[] bytes) {
        StringBuilder result = new StringBuilder();
        for (byte aByte : bytes) {
            result.append(String.format("%02x", aByte));
            // upper case
            // result.append(String.format("%02X", aByte));
        }
        return result.toString();
    }

    private void logByteArray(String tag, String msg, byte[] byteArray) {
        LOGGER.info(msg + hex(byteArray));
    }

    public byte[] put(byte[] cid, byte[] data) {
        Future<byte[]> future = executor.submit(() -> {
            String key = Base64.getEncoder().encodeToString(cid);
            logByteArray("InMemoryDatastore", "put data=", data);
            logByteArray("InMemoryDatastore", "put cid=", cid);
            store.put(key, data);
            totalBytesPut += data.length; // Increment total bytes written
            LOGGER.info("data put successfully for cid: " + key);
            return cid;
        });

        try {
            return future.get();
        } catch (Exception e) {
            // TODO: handle exception
            return null;
        }
    }

    public byte[] get(byte[] cid) {
        Future<byte[]> future = executor.submit(() -> {
            String key = Base64.getEncoder().encodeToString(cid);
            logByteArray("InMemoryDatastore", "get cid=", cid);
            if (!store.containsKey(key)){
                throw new Exception("Data not found for CID: " + key);
            }
            byte[] data = store.get(key);
            totalBytesGet += data.length; // Increment total bytes written
            logByteArray("InMemoryDatastore", "get returned data=", data);
            return data;
        });

        try {
            return future.get();
        } catch (Exception e) {
            // TODO: handle exception
            return null;
        }
    }
    // Add a method to retrieve the total bytes written
    public Long getTotalBytesPut() {
        return totalBytesPut;
    }
    // Add a method to retrieve the total bytes written
    public Long getTotalBytesGet() {
        return totalBytesGet;
    }
}
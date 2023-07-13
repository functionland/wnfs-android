package land.fx.wnfslib;

import androidx.annotation.NonNull;

public final class Config {
    private final String cid;


    public String getCid() {
        return this.cid;
    }


    public Config(String cid) {
        super();
        this.cid = cid;
    }

    public static Config create(String cid1) {
        return new Config(cid1);
    }
}
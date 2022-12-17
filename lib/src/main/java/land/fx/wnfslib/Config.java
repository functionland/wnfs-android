package land.fx.wnfslib;

import androidx.annotation.NonNull;

public final class Config {
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

    public static Config create(String cid1, String private_ref1) {
        return new Config(cid1, private_ref1);
    }
}

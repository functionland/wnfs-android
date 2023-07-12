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

    public static Config create(String cid1, String private_ref1) {
        return new Config(cid1);
    }
}


public abstract final class TypedResult<T extends Object> {
    private final T result;
    private final Boolean ok;
    private final String reason;


    public Boolean ok() {
        return this.ok;
    }
    
    public String getReason() {
        return this.reason;
    }

    public T getResult() {
        return this.result;
    }


    public TypedResult(String error, T result) {
        if (error == null){
            this.ok = true;
        } else{
            this.reason = error;
        }
        this.result = result;
    }
}


public final class Result extends TypedResult<Object> {
    public Result(String error, Object result) {
        super(error, result);
    }
    
    public static Result create(String error, Object result ) {
        return new Result(error, result);
    }
}


public final class ConfigResult extends TypedResult<Config> {
    public ConfigResult(String error, Config result) {
        super(error, result);
    }


    public static ConfigResult create(String error, Config result ) {
        return new ConfigResult(error, result);
    }
}

public final class BytesResult extends TypedResult<byte[]> {
    public BytesResult(String error, byte[] result) {
        super(error, result);
    }

    public static BytesResult create(String error, byte[] result ) {
        return new BytesResult(error, result);
    }
}

public final class StringResult extends TypedResult<String> {
    public StringResult(String error, String result) {
        super(error, result);
    }

    public static StringResult create(String error, String result ) {
        return new StringResult(error, result);
    }
}

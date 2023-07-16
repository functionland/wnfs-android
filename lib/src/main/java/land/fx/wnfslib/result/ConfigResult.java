package land.fx.wnfslib.result;

import land.fx.wnfslib.result.TypedResult;
import land.fx.wnfslib.Config;


public final class ConfigResult extends TypedResult<Config> {
    public ConfigResult(String error, Config result) {
        super(error, result);
    }


    public static ConfigResult create(String error, Config result ) {
        return new ConfigResult(error, result);
    }
}

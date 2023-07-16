package land.fx.wnfslib.result;

import land.fx.wnfslib.result.TypedResult;
import java.lang.String;


public final class StringResult extends TypedResult<String> {
    public StringResult(String error, String result) {
        super(error, result);
    }

    public static StringResult create(String error, String result ) {
        return new StringResult(error, result);
    }
}

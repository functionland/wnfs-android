package land.fx.wnfslib.result;

import land.fx.wnfslib.result.TypedResult;
import java.lang.Object;

public final class Result extends TypedResult<Object> {
    public Result(String error, Object result) {
        super(error, result);
    }
    
    public static Result create(String error, Object result ) {
        return new Result(error, result);
    }
}

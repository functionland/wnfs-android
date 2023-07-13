package land.fx.wnfslib.result;

import land.fx.wnfslib.result.TypedResult;



public final class BytesResult extends TypedResult<byte[]> {
    public BytesResult(String error, byte[] result) {
        super(error, result);
    }

    public static BytesResult create(String error, byte[] result ) {
        return new BytesResult(error, result);
    }
}

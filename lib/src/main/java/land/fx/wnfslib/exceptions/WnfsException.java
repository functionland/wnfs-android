package land.fx.wnfslib.exceptions;

import java.lang.String;
import java.lang.Exception;

public final class WnfsException extends Exception
{

    public WnfsException() {}

    // Constructor that accepts a message
    public WnfsException(String func ,String reason)
    {
        super(String.format("An Error Occured in WNFS.%s: %s", func, reason));
    }
}
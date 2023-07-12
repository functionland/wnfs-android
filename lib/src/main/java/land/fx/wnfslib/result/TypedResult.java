package land.fx.wnfslib.result;

import java.lang.Object;
import java.lang.Boolean;
import java.lang.String;

public abstract class TypedResult<T extends Object> {
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
            this.reason = null;
        } else{
            this.ok = false;
            this.reason = error;
        }
        this.result = result;
    }
}

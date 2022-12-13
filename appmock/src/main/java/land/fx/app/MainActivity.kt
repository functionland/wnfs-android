package land.fx.app

import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import land.fx.wnfslib.Fs.initRustLogger

class MainActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)
        //System.loadLibrary("wnfslib")

        //initRustLogger()
    }
}
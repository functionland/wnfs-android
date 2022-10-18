package com.functionland.app

import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import com.functionland.lib.initRustLogger

class MainActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)
        System.loadLibrary("wnfslib")

        initRustLogger()
    }
}
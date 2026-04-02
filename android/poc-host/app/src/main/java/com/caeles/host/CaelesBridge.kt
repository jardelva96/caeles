package com.caeles.host

object CaelesBridge {
    init {
        // The Android app should package and load this native library.
        // Library name maps to libcaeles_android_bridge_poc.so by default cargo naming rules.
        System.loadLibrary("caeles_android_bridge_poc")
    }

    external fun nativeHealth(): String
    external fun nativeList(registryPath: String): String
    external fun nativeRun(manifestPath: String): String
}

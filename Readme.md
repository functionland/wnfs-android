# WNFS-lib For Android

A wnfslib wrapper for Android.

## Prerequisites

Use Ubuntu (on Windows it complains about openssl)

- Kotlin toolchain
- Android SDK + NDK (latest)
- python3 and Python command available
- Cargo and CMake
- java
- gradle
- rustup target add armv7-linux-androideabi aarch64-linux-android i686-linux-android x86_64-linux-android

## Debug

Make sure you have switch to `debug` profile in cargo config, which could be found at `lib/build.gradle` 

Run the command to build

```sh
./gradlew lib:assemble
```

Connect to a device or setup an AVD and check the functionality.

```sh
./gradlew appmock:connectedCheck
```

## Build

Before make a release build, ensure you have set `profile = "release"` in cargo config.

```sh
./gradlew lib:assemble
```

The generated release build is `lib/build/outputs/aar/lib-release.aar`

## Publish New Version

Ensure you have committed your changes.

```sh
./gradlew release
```

Then simply push to the repo.

# WNFS-lib For Android

A wnfslib wrapper for Android.

## Prerequisites

- Kotlin toolchain
- Android SDK + NDK r24 (latest)

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